//! PDF OCR jobs, using Poppler and a selectable page recognizer.
mod engine;
pub mod monitor;
mod paddle;
mod store;

use std::{path::PathBuf, sync::Arc, time::Duration};

use axum::{
    body::Bytes,
    extract::{Extension, Path, Query, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use chrono::Utc;
use heramind_core::extension::ExtensionRuntime;
use redb::Database;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::{OnceCell, OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;

use crate::{
    auth_users::{SessionInfo, UserRole},
    handlers::common::{ok, HandlerResult},
    models::{ApiResponse, ErrorResponse},
    server::ServerState,
};
use engine::Engine;
use monitor::OcrMonitorSnapshot;
use paddle::Paddle;
pub use store::Key;

pub const MAX_PDF_BYTES: usize = 20 * 1024 * 1024;
const MAX_PAGES: u32 = 40;
const MAX_DOCUMENT_TEXT: usize = 1024 * 1024;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Recognizer {
    #[default]
    Tesseract,
    PaddleOcrVl,
}

impl Recognizer {
    fn job_timeout(self) -> Duration {
        match self {
            Self::Tesseract => Duration::from_secs(240),
            Self::PaddleOcrVl => Duration::from_secs(1800),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Vie,
    Eng,
    VieEng,
}

impl Language {
    const ALL: [Self; 3] = [Self::VieEng, Self::Vie, Self::Eng];

    fn engine_value(self) -> &'static str {
        match self {
            Self::Vie => "vie",
            Self::Eng => "eng",
            Self::VieEng => "vie+eng",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl JobStatus {
    fn is_active(self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct Job {
    pub id: Uuid,
    pub owner: String,
    pub filename: String,
    pub language: Language,
    #[serde(default)]
    pub engine: Recognizer,
    pub status: JobStatus,
    pub created_at: i64,
    pub finished_at: Option<i64>,
    pub pages_done: u32,
    pub page_count: Option<u32>,
    pub error_code: Option<String>,
}

impl Job {
    fn fail(&mut self, code: &str) {
        self.status = JobStatus::Failed;
        self.error_code = Some(code.to_owned());
        self.finished_at = Some(Utc::now().timestamp());
    }

    fn expired(&self, now: i64) -> bool {
        self.finished_at
            .is_some_and(|time| time + store::RETENTION_SECONDS <= now)
    }
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct Page {
    pub number: u32,
    pub text: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct JobResult {
    pub job: Job,
    pub pages: Vec<Page>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct Status {
    pub engine: Recognizer,
    pub engine_ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_loaded: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    pub languages: Vec<Language>,
    pub max_bytes: usize,
    pub max_pages: u32,
    pub retention_seconds: i64,
}

#[derive(Deserialize)]
pub struct SubmitQuery {
    pub filename: Option<String>,
    pub language: Option<Language>,
    pub engine: Option<Recognizer>,
}

#[derive(Deserialize)]
pub struct StatusQuery {
    #[serde(default)]
    pub engine: Recognizer,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateKey {
    pub name: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct IssuedKey {
    pub key: Key,
    pub token: String,
}

/// Success fields in the common API response envelope.
#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
#[aliases(
    OcrStatusResponse = Success<Status>,
    OcrJobResponse = Success<Job>,
    OcrJobListResponse = Success<Vec<Job>>,
    OcrResultResponse = Success<JobResult>,
    OcrDeletedJobResponse = Success<bool>,
    OcrMonitorResponse = Success<OcrMonitorSnapshot>,
    OcrKeyListResponse = Success<Vec<Key>>,
    OcrIssuedKeyResponse = Success<IssuedKey>,
    OcrRevokedKeyResponse = Success<bool>
)]
pub struct Success<T> {
    success: bool,
    data: T,
}

#[derive(Clone)]
pub struct Caller {
    owner: String,
    admin: bool,
}

impl Caller {
    fn owner_filter(&self) -> Option<String> {
        (!self.admin).then(|| self.owner.clone())
    }
}

pub struct OcrService {
    directory: PathBuf,
    db: OnceCell<Arc<Database>>,
    engine: Engine,
    paddle: Paddle,
    pdf_tools: OnceCell<()>,
    languages: OnceCell<Vec<Language>>,
    admitted: Arc<Semaphore>,
    workers: Semaphore,
    paddle_worker: Semaphore,
    monitor: monitor::RequestMonitor,
}

impl OcrService {
    pub fn new(directory: PathBuf, runtime: Arc<ExtensionRuntime>) -> Self {
        Self {
            directory,
            db: OnceCell::new(),
            engine: Engine::new(),
            paddle: Paddle::new(runtime),
            pdf_tools: OnceCell::new(),
            languages: OnceCell::new(),
            admitted: Arc::new(Semaphore::new(8)),
            workers: Semaphore::new(2),
            paddle_worker: Semaphore::new(1),
            monitor: monitor::RequestMonitor::new(),
        }
    }

    async fn database(&self) -> Result<&Arc<Database>, ErrorResponse> {
        self.db
            .get_or_try_init(|| async {
                let directory = self.directory.clone();
                let db = Arc::new(blocking(move || store::open(&directory)).await?);
                let weak = Arc::downgrade(&db);
                tokio::spawn(async move {
                    let mut timer = tokio::time::interval(Duration::from_secs(3600));
                    loop {
                        timer.tick().await;
                        let Some(db) = weak.upgrade() else { break };
                        let _ = blocking(move || store::prune(&db)).await;
                    }
                });
                Ok(db)
            })
            .await
    }

    async fn with_db<T: Send + 'static>(
        &self,
        callback: impl FnOnce(&Database) -> anyhow::Result<T> + Send + 'static,
    ) -> Result<T, ErrorResponse> {
        let db = self.database().await?.clone();
        blocking(move || callback(&db)).await
    }

    async fn languages(&self) -> Result<&Vec<Language>, &'static str> {
        self.languages
            .get_or_try_init(|| self.engine.languages())
            .await
    }

    async fn check_pdf_tools(&self) -> Result<&(), &'static str> {
        self.pdf_tools
            .get_or_try_init(|| self.engine.check_pdf_tools())
            .await
    }

    async fn status(&self, engine: Recognizer) -> Status {
        let pdf_ready = self.check_pdf_tools().await.is_ok();
        let (languages, model_loaded, model_version) = match engine {
            Recognizer::Tesseract => (
                self.languages().await.cloned().unwrap_or_default(),
                None,
                None,
            ),
            Recognizer::PaddleOcrVl => {
                let health = self.paddle.health().await;
                let languages = if health.available() {
                    Language::ALL.to_vec()
                } else {
                    Vec::new()
                };
                (languages, Some(health.model_loaded), health.version)
            }
        };
        Status {
            engine,
            engine_ready: pdf_ready && !languages.is_empty(),
            model_loaded,
            model_version,
            languages,
            max_bytes: MAX_PDF_BYTES,
            max_pages: MAX_PAGES,
            retention_seconds: store::RETENTION_SECONDS,
        }
    }

    async fn process(&self, mut job: Job, pdf: Bytes) {
        let _paddle_worker = match job.engine {
            Recognizer::Tesseract => None,
            Recognizer::PaddleOcrVl => {
                let Ok(permit) = self.paddle_worker.acquire().await else {
                    return;
                };
                Some(permit)
            }
        };
        let Ok(_worker) = self.workers.acquire().await else {
            return;
        };
        let result =
            tokio::time::timeout(job.engine.job_timeout(), self.extract(&mut job, pdf)).await;
        let pages = match result {
            Ok(Ok(pages)) => {
                job.status = JobStatus::Completed;
                job.finished_at = Some(Utc::now().timestamp());
                Some(pages)
            }
            Ok(Err(code)) => {
                job.fail(code);
                None
            }
            Err(_) => {
                job.fail("JOB_TIMEOUT");
                None
            }
        };
        let _ = self
            .with_db(move |db| store::save(db, &job, pages.as_deref()))
            .await;
    }

    async fn extract(&self, job: &mut Job, pdf: Bytes) -> Result<Vec<Page>, &'static str> {
        job.status = JobStatus::Running;
        self.record_progress(job).await?;
        let scratch = tempfile::Builder::new()
            .prefix("job-")
            .tempdir_in(self.directory.join("work"))
            .map_err(|_| "STORAGE_ERROR")?;
        let input = scratch.path().join("input.pdf");
        tokio::fs::write(&input, pdf)
            .await
            .map_err(|_| "STORAGE_ERROR")?;
        let page_count = self.engine.page_count(&input).await?;
        job.page_count = Some(page_count);
        self.record_progress(job).await?;
        let mut pages = Vec::with_capacity(page_count as usize);
        let mut bytes = 0;
        for number in 1..=page_count {
            let image = self.engine.render(&input, scratch.path(), number).await?;
            let text = match job.engine {
                Recognizer::Tesseract => self.engine.recognize(&image, job.language).await?,
                Recognizer::PaddleOcrVl => self.paddle.recognize(&image, job.language).await?,
            };
            if text.len() > engine::MAX_PAGE_TEXT {
                return Err("RESULT_TOO_LARGE");
            }
            let text = text.trim().to_owned();
            bytes += text.len();
            if bytes > MAX_DOCUMENT_TEXT {
                return Err("RESULT_TOO_LARGE");
            }
            tokio::fs::remove_file(image)
                .await
                .map_err(|_| "STORAGE_ERROR")?;
            pages.push(Page { number, text });
            job.pages_done = number;
            self.record_progress(job).await?;
        }
        Ok(pages)
    }

    async fn record_progress(&self, job: &Job) -> Result<(), &'static str> {
        let snapshot = job.clone();
        self.with_db(move |db| store::save(db, &snapshot, None))
            .await
            .map_err(|_| "STORAGE_ERROR")
    }
}

async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
) -> Result<T, ErrorResponse> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|error| {
            tracing::error!(category = "ocr", %error, "OCR storage task stopped");
            ErrorResponse::internal("OCR storage unavailable")
        })?
        .map_err(|error| {
            tracing::error!(category = "ocr", %error, "OCR storage operation failed");
            ErrorResponse::internal("OCR storage unavailable")
        })
}

fn digest(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub async fn authenticate(
    State(state): State<ServerState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ErrorResponse> {
    let headers = request.headers();
    let caller = if let Some(raw) = headers.get("x-ocr-key") {
        let token = raw
            .to_str()
            .ok()
            .filter(|token| token.starts_with("ocr_") && token.len() == 68)
            .ok_or_else(|| ErrorResponse::unauthorized("Invalid OCR key"))?;
        let digest = digest(token);
        let service = request
            .extensions()
            .get::<Arc<OcrService>>()
            .ok_or_else(|| ErrorResponse::internal("OCR service unavailable"))?;
        let key = service
            .with_db(move |db| store::key_by_digest(db, &digest))
            .await?
            .ok_or_else(|| ErrorResponse::unauthorized("Invalid OCR key"))?;
        Caller {
            owner: format!("integration:{}", key.id),
            admin: false,
        }
    } else {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| ErrorResponse::unauthorized("Use a HeraMind session or X-OCR-Key"))?;
        let session = state
            .auth
            .user_state
            .validate_token(token)
            .map_err(|_| ErrorResponse::unauthorized("Invalid HeraMind session"))?;
        Caller {
            owner: format!("user:{}", session.user_id),
            admin: session.role == UserRole::Admin,
        }
    };
    request.extensions_mut().insert(caller);
    if request.method() == Method::POST && request.uri().path() == "/api/ocr/jobs" {
        let service = request
            .extensions()
            .get::<Arc<OcrService>>()
            .ok_or_else(|| ErrorResponse::internal("OCR service unavailable"))?;
        let permit = service
            .admitted
            .clone()
            .try_acquire_owned()
            .map_err(|_| queue_full())?;
        request.extensions_mut().insert(Arc::new(permit));
        return tokio::time::timeout(Duration::from_secs(60), next.run(request))
            .await
            .map_err(|_| {
                ErrorResponse::new(
                    "OCR_UPLOAD_TIMEOUT",
                    "OCR upload timed out",
                    StatusCode::REQUEST_TIMEOUT,
                )
            });
    }
    Ok(next.run(request).await)
}

pub async fn no_store(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// Check OCR engine availability
///
/// Use `GET /api/ocr/status?engine=paddle_ocr_vl` before submitting a PDF to
/// PaddleOCR-VL. Submit when `data.engine_ready` is true and `data.languages`
/// includes the requested language. A ready backend can load the model on the
/// first request when `data.model_loaded` is false.
#[utoipa::path(get, path = "/api/ocr/status", tag = "OCR",
    security(("ocrKey" = []), ("ocrSession" = [])),
    params(("engine" = Option<Recognizer>, Query,
        description = "Use paddle_ocr_vl to check the configured PaddleOCR-VL backend. Defaults to tesseract.",
        example = "paddle_ocr_vl")),
    responses((status = 200, description = "OCR engine status", body = OcrStatusResponse),
        (status = 401, description = "Authentication required")))]
pub async fn status(
    Extension(service): Extension<Arc<OcrService>>,
    Query(query): Query<StatusQuery>,
) -> HandlerResult<Status> {
    ok(service.status(query.engine).await)
}

/// Submit a PDF for OCR
///
/// Use `POST /api/ocr/jobs?engine=paddle_ocr_vl&language=vie_eng` with
/// `Content-Type: application/pdf` and the PDF bytes as the request body.
/// PaddleOCR-VL runs on NVIDIA GPU when the configured inference backend uses
/// `PADDLE_DEVICE=gpu:0`. The server controls device selection.
///
/// Send an OCR integration key in `X-OCR-Key` or a HeraMind bearer session.
/// After `202 Accepted`, poll the URL in the `Location` response header with
/// the same credential to retrieve the recognized text.
#[utoipa::path(post, path = "/api/ocr/jobs", tag = "OCR",
    security(("ocrKey" = []), ("ocrSession" = [])),
    params(("filename" = Option<String>, Query, description = "Optional display filename"),
        ("language" = Option<Language>, Query, description = "Default: vie_eng. PaddleOCR-VL treats this as a hint."),
        ("engine" = Option<Recognizer>, Query,
            description = "Use paddle_ocr_vl to send pages to the configured PaddleOCR-VL backend. Defaults to tesseract.",
            example = "paddle_ocr_vl")),
    request_body(content = Vec<u8>, content_type = "application/pdf", description = "Raw PDF bytes, up to 20 MiB and 40 pages."),
    responses((status = 202, description = "Job accepted", body = OcrJobResponse,
        headers(("Location" = String, description = "Path to the OCR job"))),
        (status = 400, description = "Invalid PDF header or query"),
        (status = 401, description = "Authentication required"),
        (status = 408, description = "Upload timed out"),
        (status = 415, description = "Use Content-Type: application/pdf"),
        (status = 413, description = "PDF exceeds upload limit"),
        (status = 429, description = "OCR queue is full"),
        (status = 503, description = "OCR engine is unavailable")))]
pub async fn submit(
    Extension(service): Extension<Arc<OcrService>>,
    Extension(caller): Extension<Caller>,
    Extension(permit): Extension<Arc<OwnedSemaphorePermit>>,
    Query(query): Query<SubmitQuery>,
    headers: HeaderMap,
    pdf: Bytes,
) -> Result<
    (
        StatusCode,
        [(header::HeaderName, String); 1],
        Json<ApiResponse<Job>>,
    ),
    ErrorResponse,
> {
    let is_pdf = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/pdf"));
    if !is_pdf {
        return Err(ErrorResponse::new(
            "PDF_REQUIRED",
            "Use Content-Type: application/pdf",
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ));
    }
    if !pdf.starts_with(b"%PDF-") {
        return Err(ErrorResponse::bad_request("A PDF file is required"));
    }

    let language = query.language.unwrap_or(Language::VieEng);
    let engine = query.engine.unwrap_or_default();
    let status = service.status(engine).await;
    if !status.engine_ready {
        return Err(ErrorResponse::service_unavailable(
            "OCR engine is unavailable",
        ));
    }
    if !status.languages.contains(&language) {
        return Err(ErrorResponse::service_unavailable(
            "OCR language is unavailable",
        ));
    }
    let filename: String = query
        .filename
        .as_deref()
        .unwrap_or("document.pdf")
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|character| !character.is_control())
        .take(160)
        .collect();
    let job = Job {
        id: Uuid::new_v4(),
        owner: caller.owner,
        filename: if filename.trim().is_empty() {
            "document.pdf".into()
        } else {
            filename
        },
        language,
        engine,
        status: JobStatus::Queued,
        created_at: Utc::now().timestamp(),
        finished_at: None,
        pages_done: 0,
        page_count: None,
        error_code: None,
    };
    let accepted = job.clone();
    let task = tokio::spawn(async move {
        if !service
            .with_db(move |db| store::submit(db, &accepted))
            .await?
        {
            return Err(queue_full());
        }
        let response = job.clone();
        tokio::spawn(async move {
            let _permit = permit;
            service.process(job, pdf).await;
        });
        Ok(response)
    });
    // Admission and worker creation continue if the HTTP caller disconnects.
    let job = task
        .await
        .map_err(|_| ErrorResponse::internal("OCR submission failed"))??;
    Ok((
        StatusCode::ACCEPTED,
        [(header::LOCATION, format!("/api/ocr/jobs/{}", job.id))],
        ok(job)?,
    ))
}

fn queue_full() -> ErrorResponse {
    ErrorResponse::new(
        "OCR_QUEUE_FULL",
        "OCR queue is full. Try again later.",
        StatusCode::TOO_MANY_REQUESTS,
    )
}

#[utoipa::path(get, path = "/api/ocr/jobs", tag = "OCR",
    security(("ocrKey" = []), ("ocrSession" = [])),
    responses((status = 200, description = "Most recent 50 jobs visible to this caller", body = OcrJobListResponse),
        (status = 401, description = "Authentication required")))]
pub async fn list(
    Extension(service): Extension<Arc<OcrService>>,
    Extension(caller): Extension<Caller>,
) -> HandlerResult<Vec<Job>> {
    let owner = caller.owner_filter();
    ok(service
        .with_db(move |db| store::list(db, owner.as_deref()))
        .await?)
}

/// Read an OCR job and its pages
///
/// Poll while `data.job.status` is `queued` or `running`. When it is
/// `completed`, read the recognized text from `data.pages[].text`. When it is
/// `failed`, read `data.job.error_code`.
#[utoipa::path(get, path = "/api/ocr/jobs/{id}", tag = "OCR",
    security(("ocrKey" = []), ("ocrSession" = [])),
    params(("id" = Uuid, Path, description = "OCR job ID")),
    responses((status = 200, description = "Job status and completed pages", body = OcrResultResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Job missing, expired, or owned by another caller")))]
pub async fn get(
    Extension(service): Extension<Arc<OcrService>>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<Uuid>,
) -> HandlerResult<JobResult> {
    let owner = caller.owner_filter();
    let (job, pages) = service
        .with_db(move |db| store::get(db, id, owner.as_deref()))
        .await?
        .ok_or_else(|| ErrorResponse::not_found("OCR job"))?;
    ok(JobResult { job, pages })
}

/// Delete a finished OCR job and its extracted text.
#[utoipa::path(delete, path = "/api/ocr/jobs/{id}", tag = "OCR",
    security(("ocrKey" = []), ("ocrSession" = [])),
    params(("id" = Uuid, Path, description = "OCR job ID")),
    responses((status = 200, description = "Job and extracted text deleted", body = OcrDeletedJobResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Job missing, expired, or owned by another caller"),
        (status = 409, description = "Job is queued or running")))]
pub async fn delete_job(
    Extension(service): Extension<Arc<OcrService>>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<Uuid>,
) -> HandlerResult<bool> {
    let owner = caller.owner_filter();
    match service
        .with_db(move |db| store::delete_job(db, id, owner.as_deref()))
        .await?
    {
        store::DeleteJobOutcome::Deleted => ok(true),
        store::DeleteJobOutcome::NotFound => Err(ErrorResponse::not_found("OCR job")),
        store::DeleteJobOutcome::Active => {
            Err(ErrorResponse::conflict("OCR job is still processing"))
        }
    }
}

fn require_admin(session: &SessionInfo) -> Result<(), ErrorResponse> {
    if session.role != UserRole::Admin {
        return Err(ErrorResponse::forbidden(
            "OCR key administration requires an administrator",
        ));
    }
    Ok(())
}

#[utoipa::path(get, path = "/api/ocr/keys", tag = "OCR",
    security(("ocrSession" = [])),
    responses((status = 200, description = "List OCR integration keys without secrets", body = OcrKeyListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Administrator required")))]
pub async fn keys(
    Extension(service): Extension<Arc<OcrService>>,
    Extension(session): Extension<SessionInfo>,
) -> HandlerResult<Vec<Key>> {
    require_admin(&session)?;
    ok(service.with_db(store::list_keys).await?)
}

#[utoipa::path(post, path = "/api/ocr/keys", tag = "OCR", request_body = CreateKey,
    security(("ocrSession" = [])),
    responses((status = 200, description = "Issue an OCR-only key. Token is shown once.", body = OcrIssuedKeyResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Administrator required")))]
pub async fn issue(
    Extension(service): Extension<Arc<OcrService>>,
    Extension(session): Extension<SessionInfo>,
    Json(input): Json<CreateKey>,
) -> HandlerResult<IssuedKey> {
    require_admin(&session)?;
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) {
        return Err(ErrorResponse::validation(
            "OCR key name must contain 1–80 characters",
        ));
    }
    let token = format!("ocr_{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let digest = digest(&token);
    let key = Key {
        id: Uuid::new_v4(),
        name: name.to_owned(),
        created_at: Utc::now().timestamp(),
    };
    let entry = key.clone();
    if !service
        .with_db(move |db| store::insert_key(db, &digest, &entry))
        .await?
    {
        return Err(ErrorResponse::conflict("OCR key limit reached"));
    }
    ok(IssuedKey { key, token })
}

#[utoipa::path(delete, path = "/api/ocr/keys/{id}", tag = "OCR",
    security(("ocrSession" = [])),
    params(("id" = Uuid, Path, description = "OCR key ID")),
    responses((status = 200, description = "Key revoked", body = OcrRevokedKeyResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Administrator required"),
        (status = 404, description = "Key missing")))]
pub async fn revoke(
    Extension(service): Extension<Arc<OcrService>>,
    Extension(session): Extension<SessionInfo>,
    Path(id): Path<Uuid>,
) -> HandlerResult<bool> {
    require_admin(&session)?;
    if !service.with_db(move |db| store::revoke_key(db, id)).await? {
        return Err(ErrorResponse::not_found("OCR key"));
    }
    ok(true)
}
