use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Instant,
};

use axum::{
    extract::{Extension, Request},
    http::{header, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use super::{Job, JobStatus, Language, OcrService, Recognizer};
use crate::{auth_users::SessionInfo, handlers::common::HandlerResult};

const MAX_REQUESTS: usize = 1000;
const WINDOW_MS: i64 = 24 * 60 * 60 * 1000;
const JOB_PATH: &str = "/api/ocr/jobs/";

#[derive(Clone, Copy, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Route {
    Status,
    Submit,
    ListJobs,
    GetJob,
}

#[derive(Clone, Copy, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Authentication {
    OcrKey,
    Session,
    None,
}

#[derive(Clone, Serialize, utoipa::ToSchema)]
pub(crate) struct RequestSample {
    sequence: u64,
    at_ms: i64,
    duration_ms: u64,
    route: Route,
    authentication: Authentication,
    status: u16,
    job_id: Option<Uuid>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub(crate) struct JobSummary {
    id: Uuid,
    engine: Recognizer,
    language: Language,
    status: JobStatus,
    created_at: i64,
    finished_at: Option<i64>,
    pages_done: u32,
    page_count: Option<u32>,
    error_code: Option<String>,
}

impl From<Job> for JobSummary {
    fn from(job: Job) -> Self {
        Self {
            id: job.id,
            engine: job.engine,
            language: job.language,
            status: job.status,
            created_at: job.created_at,
            finished_at: job.finished_at,
            pages_done: job.pages_done,
            page_count: job.page_count,
            error_code: job.error_code,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct OcrMonitorSnapshot {
    /// Unix milliseconds.
    generated_at_ms: i64,
    /// Earliest covered request timestamp in Unix milliseconds.
    observed_since_ms: i64,
    max_requests: usize,
    requests: Vec<RequestSample>,
    jobs: Vec<JobSummary>,
}

#[derive(Default)]
struct Buffer {
    requests: VecDeque<RequestSample>,
    dropped_through_ms: Option<i64>,
}

pub(super) struct RequestMonitor {
    started_at_ms: i64,
    buffer: Mutex<Buffer>,
}

impl RequestMonitor {
    pub(super) fn new() -> Self {
        Self {
            started_at_ms: Utc::now().timestamp_millis(),
            buffer: Mutex::new(Buffer::default()),
        }
    }

    fn record(&self, mut sample: RequestSample) {
        let mut buffer = self
            .buffer
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        sample.sequence = buffer.requests.back().map_or(1, |last| last.sequence + 1);
        if buffer.requests.len() == MAX_REQUESTS {
            if let Some(dropped) = buffer.requests.pop_front() {
                buffer.dropped_through_ms = Some(
                    buffer
                        .dropped_through_ms
                        .map_or(dropped.at_ms, |last| last.max(dropped.at_ms)),
                );
            }
        }
        buffer.requests.push_back(sample);
    }

    fn snapshot(&self, jobs: Vec<Job>) -> OcrMonitorSnapshot {
        let buffer = self
            .buffer
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let now = Utc::now().timestamp_millis();
        OcrMonitorSnapshot {
            generated_at_ms: now,
            observed_since_ms: self
                .started_at_ms
                .max(now - WINDOW_MS)
                .max(buffer.dropped_through_ms.unwrap_or(self.started_at_ms))
                .min(now),
            max_requests: MAX_REQUESTS,
            requests: buffer
                .requests
                .iter()
                .rev()
                .filter(|request| request.at_ms >= now - WINDOW_MS)
                .cloned()
                .collect(),
            jobs: jobs.into_iter().map(JobSummary::from).collect(),
        }
    }
}

pub async fn record_request(
    Extension(service): Extension<Arc<OcrService>>,
    request: Request,
    next: Next,
) -> Response {
    let (route, job_id) = match (request.method(), request.uri().path()) {
        (&Method::GET, "/api/ocr/status") => (Route::Status, None),
        (&Method::POST, "/api/ocr/jobs") => (Route::Submit, None),
        (&Method::GET, "/api/ocr/jobs") => (Route::ListJobs, None),
        (&Method::GET, path) if path.starts_with(JOB_PATH) => (
            Route::GetJob,
            path.strip_prefix(JOB_PATH)
                .and_then(|id| Uuid::parse_str(id).ok()),
        ),
        _ => return next.run(request).await,
    };
    let authentication = if request.headers().contains_key("x-ocr-key") {
        Authentication::OcrKey
    } else if request.headers().contains_key(header::AUTHORIZATION) {
        Authentication::Session
    } else {
        Authentication::None
    };
    let started = Instant::now();
    let response = next.run(request).await;
    let job_id = if matches!(route, Route::Submit) && response.status() == StatusCode::ACCEPTED {
        response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|path| path.strip_prefix(JOB_PATH))
            .and_then(|id| Uuid::parse_str(id).ok())
    } else {
        job_id
    };
    service.monitor.record(RequestSample {
        sequence: 0,
        at_ms: Utc::now().timestamp_millis(),
        duration_ms: started.elapsed().as_millis() as u64,
        route,
        authentication,
        status: response.status().as_u16(),
        job_id,
    });
    response
}

#[utoipa::path(get, path = "/api/ocr/monitor", tag = "OCR",
    security(("ocrSession" = [])),
    responses((status = 200, description = "Recent OCR HTTP requests and job metadata", body = OcrMonitorResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Administrator required")))]
pub async fn snapshot(
    Extension(service): Extension<Arc<OcrService>>,
    Extension(session): Extension<SessionInfo>,
) -> HandlerResult<OcrMonitorSnapshot> {
    super::require_admin(&session)?;
    let jobs = service.with_db(|db| super::store::list(db, None)).await?;
    super::ok(service.monitor.snapshot(jobs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_caps_requests_and_exposes_the_coverage_boundary() {
        let mut monitor = RequestMonitor::new();
        let first_at = Utc::now().timestamp_millis() - 2_000;
        monitor.started_at_ms = first_at;
        for index in 0..MAX_REQUESTS + 2 {
            monitor.record(RequestSample {
                sequence: 0,
                at_ms: first_at + index as i64,
                duration_ms: 2,
                route: Route::Submit,
                authentication: Authentication::OcrKey,
                status: 202,
                job_id: None,
            });
        }
        let snapshot = monitor.snapshot(Vec::new());
        assert_eq!(snapshot.requests.len(), MAX_REQUESTS);
        assert_eq!(snapshot.observed_since_ms, first_at + 1);
        assert_eq!(
            snapshot.requests.first().unwrap().sequence,
            MAX_REQUESTS as u64 + 2
        );
        assert_eq!(snapshot.requests.last().unwrap().sequence, 3);
    }
}
