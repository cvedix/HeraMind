//! Optional, per-installation OpenClaw service. No global npm/Node install,
//! shell interpolation, public listener, or credentials in API responses.
use crate::{
    auth_users::{SessionInfo, UserRole},
    handlers::common::{ok, HandlerResult},
    models::ErrorResponse,
    server::ServerState,
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio::{
    process::{Child, Command},
    sync::Mutex,
};

const NODE_VERSION: &str = "24.22.2";
const OPENCLAW_VERSION: &str = "2026.10.8";
const ENDPOINT: &str = "http://127.0.0.1:19376/v1";
const LAUNCHER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scripts/agentrouter-openclaw.mjs"
));
const TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../deploy/agentrouter-openclaw.config.json"
));

#[derive(Clone, Deserialize, Serialize)]
struct Settings {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "default_model")]
    model: String,
    #[serde(default = "default_protocol")]
    protocol: String,
    #[serde(default)]
    backend_id: Option<String>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            model: default_model(),
            protocol: default_protocol(),
            backend_id: None,
        }
    }
}
fn default_model() -> String {
    "gpt-6.1-sol".into()
}
fn default_protocol() -> String {
    "openai-responses".into()
}

#[derive(Default, Deserialize, Serialize)]
struct Credentials {
    #[serde(rename = "apiKey")]
    api_key: String,
    #[serde(rename = "gatewayToken")]
    gateway_token: String,
}

#[derive(Clone, Default, Serialize)]
struct Progress {
    busy: bool,
    phase: String,
    error: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ConfigureRequest {
    pub name: String,
    pub model: String,
    pub protocol: String,
    pub api_key: Option<String>,
    pub backend_id: Option<String>,
}

pub struct AgentRouterService {
    root: PathBuf,
    operation: Arc<Mutex<()>>,
    progress: Mutex<Progress>,
    child: Mutex<Option<Child>>,
}
static SERVICE: OnceLock<Arc<AgentRouterService>> = OnceLock::new();
fn service() -> Arc<AgentRouterService> {
    SERVICE
        .get_or_init(|| {
            Arc::new(AgentRouterService::new(
                heramind_core::paths::data_dir().join("agentrouter-openclaw"),
            ))
        })
        .clone()
}
impl AgentRouterService {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            operation: Arc::new(Mutex::new(())),
            progress: Mutex::new(Progress::default()),
            child: Mutex::new(None),
        }
    }
    fn node(&self) -> PathBuf {
        self.root.join(if cfg!(windows) {
            "node-runtime/node.exe"
        } else {
            "node-runtime/bin/node"
        })
    }
    fn cli(&self) -> PathBuf {
        self.root.join("client/node_modules/openclaw/openclaw.mjs")
    }
    fn settings(&self) -> anyhow::Result<Settings> {
        read_or_default(&self.root.join("service.json"))
    }
    fn credentials(&self) -> anyhow::Result<Credentials> {
        read_or_default(&self.root.join("secrets.json"))
    }
    fn installed(&self) -> bool {
        self.node().is_file()
            && self.cli().is_file()
            && std::fs::read(self.root.join("client/node_modules/openclaw/package.json"))
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                .is_some_and(|p| p["version"] == OPENCLAW_VERSION)
    }
    async fn phase(&self, phase: &str) {
        let mut progress = self.progress.lock().await;
        progress.phase = phase.into();
        progress.error = None;
    }
    async fn managed(&self) -> bool {
        let mut child = self.child.lock().await;
        if child
            .as_mut()
            .is_some_and(|c| !matches!(c.try_wait(), Ok(None)))
        {
            *child = None;
        }
        child.is_some()
    }
    async fn running(&self) -> bool {
        let Ok(credentials) = self.credentials() else {
            return false;
        };
        if credentials.gateway_token.is_empty() {
            return false;
        }
        let Ok(client) = http_client(2) else {
            return false;
        };
        client
            .get(format!("{ENDPOINT}/models"))
            .bearer_auth(credentials.gateway_token)
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
    }
    async fn status(&self) -> anyhow::Result<Value> {
        let settings = self.settings()?;
        let credentials = self.credentials()?;
        let manager = heramind_agent::llm_backends::get_instance_manager().ok();
        let backend_id = settings.backend_id.or_else(|| {
            manager.as_ref().and_then(|manager| {
                manager
                    .list_instances()
                    .into_iter()
                    .find(|b| {
                        b.endpoint.as_deref() == Some(ENDPOINT)
                            && b.api_key.as_deref() == Some(credentials.gateway_token.as_str())
                    })
                    .map(|b| b.id)
            })
        });
        let mut progress = self.progress.lock().await.clone();
        progress.busy = self.operation.try_lock().is_err();
        Ok(
            json!({ "supported": platform_archive().is_ok(), "installed": self.installed(),
            "running": self.running().await, "managed": self.managed().await, "enabled": settings.enabled,
            "configured": !credentials.api_key.is_empty(), "model": settings.model, "protocol": settings.protocol,
            "backend_id": backend_id, "endpoint": ENDPOINT, "node_version": NODE_VERSION,
            "openclaw_version": OPENCLAW_VERSION, "progress": progress }),
        )
    }
    fn write_resources(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.root.join("workspace"))?;
        private_write(&self.root.join("launcher.mjs"), LAUNCHER.as_bytes())?;
        let settings = self.settings()?;
        let config = make_config(&settings.model, &settings.protocol)?;
        private_write(
            &self.root.join("config.template.json"),
            &serde_json::to_vec_pretty(&config)?,
        )
    }
    async fn install(&self) -> anyhow::Result<()> {
        platform_archive()?;
        std::fs::create_dir_all(&self.root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.root, std::fs::Permissions::from_mode(0o700))?;
        }
        self.phase("runtime").await;
        let node_ok = if self.node().is_file() {
            Command::new(self.node())
                .arg("--version")
                .output()
                .await
                .is_ok_and(|out| {
                    out.status.success()
                        && String::from_utf8_lossy(&out.stdout).trim() == format!("v{NODE_VERSION}")
                })
        } else {
            false
        };
        if !node_ok {
            if self.running().await {
                anyhow::bail!("Stop the OpenClaw service before replacing its runtime");
            }
            let (archive, prefix) = platform_archive()?;
            let client = http_client(180)?;
            let checksums = client
                .get(format!(
                    "https://nodejs.org/dist/v{NODE_VERSION}/SHASUMS256.txt"
                ))
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?;
            let expected = checksum_for(&checksums, &archive)?;
            let bytes = client
                .get(format!("https://nodejs.org/dist/v{NODE_VERSION}/{archive}"))
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;
            if format!("{:x}", Sha256::digest(&bytes)) != expected {
                anyhow::bail!("Node.js archive checksum verification failed");
            }
            let target = self.root.clone();
            tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                let stage = tempfile::tempdir_in(&target)?;
                extract_runtime(&bytes, stage.path(), &prefix, archive.ends_with(".zip"))?;
                replace_directory(&stage.path().join(prefix), &target.join("node-runtime"))
            })
            .await??;
        }
        self.phase("openclaw").await;
        if !self.installed() {
            let npm = [
                self.root
                    .join("node-runtime/lib/node_modules/npm/bin/npm-cli.js"),
                self.root
                    .join("node-runtime/node_modules/npm/bin/npm-cli.js"),
                self.root.join("client/node_modules/npm/bin/npm-cli.js"),
            ]
            .into_iter()
            .find(|p| p.is_file())
            .ok_or_else(|| anyhow::anyhow!("The private Node.js runtime does not include npm"))?;
            let stage = tempfile::tempdir_in(&self.root)?;
            private_write(
                &self.root.join("npmrc"),
                b"registry=https://registry.npmjs.org/\n",
            )?;
            let mut command = Command::new(self.node());
            let path = std::env::join_paths(
                std::iter::once(self.node().parent().unwrap().to_path_buf()).chain(
                    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
                ),
            )?;
            command
                .arg(npm)
                .args(["install", "--prefix"])
                .arg(stage.path())
                .args([
                    "--save-exact",
                    &format!("openclaw@{OPENCLAW_VERSION}"),
                    "--no-audit",
                    "--no-fund",
                ])
                .env("PATH", path)
                .env("npm_config_userconfig", self.root.join("npmrc"))
                .env("npm_config_cache", self.root.join("npm-cache"))
                .env_remove("HERAMIND_AGENTROUTER_KEY")
                .env_remove("HERAMIND_OPENCLAW_GATEWAY_TOKEN")
                .kill_on_drop(true);
            let output = tokio::time::timeout(Duration::from_secs(900), command.output())
                .await
                .map_err(|_| {
                    anyhow::anyhow!(
                        "OpenClaw installation timed out; retry when the network is available"
                    )
                })??;
            if !output.status.success() {
                anyhow::bail!("OpenClaw npm installation failed. Check outbound access to registry.npmjs.org and available disk space.");
            }
            replace_directory(stage.path(), &self.root.join("client"))?;
        }
        self.write_resources()?;
        self.phase("ready").await;
        Ok(())
    }
    async fn start(&self) -> anyhow::Result<()> {
        if self.running().await {
            self.phase("running").await;
            return Ok(());
        }
        anyhow::ensure!(self.installed(), "Install OpenClaw first");
        let credentials = self.credentials()?;
        anyhow::ensure!(
            !credentials.api_key.is_empty() && !credentials.gateway_token.is_empty(),
            "Configure the AgentRouter API key and model first"
        );
        if tokio::net::TcpStream::connect(("127.0.0.1", 19376))
            .await
            .is_ok()
        {
            anyhow::bail!("Port 19376 is occupied by another service");
        }
        self.write_resources()?;
        self.phase("starting").await;
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.root.join("service.log"))?;
        let mut command = Command::new(self.node());
        #[cfg(unix)] {
            use std::os::unix::process::CommandExt;
            command.as_std_mut().process_group(0);
        }
        let child = command
            .arg(self.root.join("launcher.mjs"))
            .arg("start")
            .env("HERAMIND_AGENTROUTER_DATA_DIR", &self.root)
            .env(
                "HERAMIND_OPENCLAW_CONFIG_TEMPLATE",
                self.root.join("config.template.json"),
            )
            .env("HERAMIND_BRIDGE_PARENT_PID", std::process::id().to_string())
            .env_remove("HERAMIND_AGENTROUTER_KEY")
            .env_remove("HERAMIND_OPENCLAW_GATEWAY_TOKEN")
            .stdout(log.try_clone()?)
            .stderr(log)
            .kill_on_drop(true)
            .spawn()?;
        *self.child.lock().await = Some(child);
        for _ in 0..60 {
            if self.running().await {
                self.phase("running").await;
                return Ok(());
            }
            if !self.managed().await {
                anyhow::bail!("OpenClaw exited during startup. Check the local service log.");
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        self.stop_child().await?;
        anyhow::bail!("OpenClaw startup timed out")
    }
    async fn stop_child(&self) -> anyhow::Result<()> {
        if let Some(mut child) = self.child.lock().await.take() {
            #[cfg(unix)]
            {
                if let Some(id) = child.id() {
                    let _ = Command::new("/bin/kill")
                        .args(["-TERM", "--", &format!("-{id}")])
                        .status()
                        .await;
                }
            }
            #[cfg(windows)]
            {
                if let Some(id) = child.id() {
                    let _ = Command::new("taskkill").args(["/PID", &id.to_string(), "/T", "/F"]).status().await;
                }
            }
            if tokio::time::timeout(Duration::from_secs(12), child.wait())
                .await
                .is_err()
            {
                #[cfg(unix)] if let Some(id) = child.id() {
                    let _ = Command::new("/bin/kill").args(["-KILL", "--", &format!("-{id}")]).status().await;
                }
                child.kill().await?;
            }
        }
        Ok(())
    }
}

fn read_or_default<T: serde::de::DeserializeOwned + Default>(path: &Path) -> anyhow::Result<T> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(e.into()),
    }
}
fn private_write(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let mut file = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|e| e.error)?;
    Ok(())
}
fn http_client(timeout: u64) -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .redirect(reqwest::redirect::Policy::none())
        .build()?)
}
fn validate_input(model: &str, protocol: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !model.trim().is_empty() && model.len() <= 512 && !model.chars().any(char::is_control),
        "Enter a valid model ID"
    );
    anyhow::ensure!(
        [
            "openai-responses",
            "openai-completions",
            "anthropic-messages"
        ]
        .contains(&protocol),
        "Unsupported provider protocol"
    );
    Ok(())
}
fn make_config(model: &str, protocol: &str) -> anyhow::Result<Value> {
    validate_input(model, protocol)?;
    let mut config: Value = serde_json::from_str(TEMPLATE)?;
    let provider = &mut config["models"]["providers"]["agentrouter"];
    provider["api"] = json!(protocol);
    provider["baseUrl"] = json!(if protocol == "anthropic-messages" {
        "https://agentrouter.org"
    } else {
        "https://agentrouter.org/v1"
    });
    provider["models"][0]["id"] = json!(model.trim());
    provider["models"][0]["name"] = json!(model.trim());
    config["agents"]["defaults"]["model"]["primary"] =
        json!(format!("agentrouter/{}", model.trim()));
    Ok(config)
}
fn platform_archive() -> anyhow::Result<(String, String)> {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "win",
        _ => anyhow::bail!("OpenClaw setup is supported on macOS, Linux and Windows"),
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        _ => anyhow::bail!("OpenClaw setup requires x64 or ARM64"),
    };
    let prefix = format!("node-v{NODE_VERSION}-{os}-{arch}");
    Ok((
        format!("{prefix}.{}", if os == "win" { "zip" } else { "tar.gz" }),
        prefix,
    ))
}
fn checksum_for(checksums: &str, name: &str) -> anyhow::Result<String> {
    checksums
        .lines()
        .find_map(|line| {
            let mut fields = line.split_whitespace();
            let hash = fields.next()?;
            let file = fields.next()?;
            (file == name && hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
                .then(|| hash.to_ascii_lowercase())
        })
        .ok_or_else(|| anyhow::anyhow!("Node.js release checksum is missing"))
}
fn extract_runtime(
    bytes: &[u8],
    destination: &Path,
    prefix: &str,
    zip: bool,
) -> anyhow::Result<()> {
    if zip {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let path = entry
                .enclosed_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid runtime archive path"))?;
            anyhow::ensure!(path.starts_with(prefix), "Unexpected runtime archive root");
            let target = destination.join(path);
            if entry.is_dir() {
                std::fs::create_dir_all(target)?;
            } else {
                std::fs::create_dir_all(target.parent().unwrap())?;
                let mut file = std::fs::File::create(target)?;
                std::io::copy(&mut entry, &mut file)?;
            }
        }
    } else {
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
        for entry in archive.entries()? {
            let mut entry = entry?;
            anyhow::ensure!(
                entry.path()?.starts_with(prefix),
                "Unexpected runtime archive root"
            );
            anyhow::ensure!(
                entry.unpack_in(destination)?,
                "Invalid runtime archive path"
            );
        }
    }
    Ok(())
}
fn replace_directory(source: &Path, target: &Path) -> anyhow::Result<()> {
    let previous = target.with_extension("previous");
    if previous.exists() {
        std::fs::remove_dir_all(&previous)?;
    }
    if target.exists() {
        std::fs::rename(target, &previous)?;
    }
    if let Err(error) = std::fs::rename(source, target) {
        if previous.exists() {
            let _ = std::fs::rename(&previous, target);
        }
        return Err(error.into());
    }
    if previous.exists() {
        std::fs::remove_dir_all(previous)?;
    }
    Ok(())
}
fn require_admin(session: &SessionInfo) -> Result<(), ErrorResponse> {
    if session.role != UserRole::Admin {
        return Err(ErrorResponse::new(
            "FORBIDDEN",
            "Admin access required to manage OpenClaw",
            StatusCode::FORBIDDEN,
        ));
    }
    Ok(())
}
fn failure(error: anyhow::Error) -> ErrorResponse {
    ErrorResponse::bad_request(error.to_string())
}

#[utoipa::path(get, path = "/api/agentrouter-service", tag = "llm-backends", responses((status = 200, description = "Optional OpenClaw service status")))]
pub async fn status_handler(Extension(admin): Extension<SessionInfo>) -> HandlerResult<Value> {
    require_admin(&admin)?;
    ok(service().status().await.map_err(failure)?)
}
#[utoipa::path(post, path = "/api/agentrouter-service/install", tag = "llm-backends", responses((status = 200, description = "Installation accepted; poll service progress")))]
pub async fn install_handler(Extension(admin): Extension<SessionInfo>) -> HandlerResult<Value> {
    require_admin(&admin)?;
    let service = service();
    let permit = service
        .operation
        .clone()
        .try_lock_owned()
        .map_err(|_| ErrorResponse::conflict("An OpenClaw operation is already running"))?;
    *service.progress.lock().await = Progress {
        busy: true,
        phase: "queued".into(),
        error: None,
    };
    tokio::spawn(async move {
        let _permit = permit;
        let result = service.install().await;
        let mut progress = service.progress.lock().await;
        progress.busy = false;
        if let Err(error) = result {
            progress.phase = "failed".into();
            progress.error = Some(error.to_string());
        }
    });
    ok(json!({"accepted": true}))
}
#[utoipa::path(post, path = "/api/agentrouter-service/configure", tag = "llm-backends", request_body = ConfigureRequest, responses((status = 200, description = "Configure the shared gateway and a HeraMind backend")))]
pub async fn configure_handler(
    State(state): State<ServerState>,
    Extension(admin): Extension<SessionInfo>,
    Json(request): Json<ConfigureRequest>,
) -> HandlerResult<Value> {
    require_admin(&admin)?;
    validate_input(&request.model, &request.protocol).map_err(failure)?;
    if request.name.trim().is_empty() {
        return Err(ErrorResponse::bad_request("Enter a backend name"));
    }
    let service = service();
    let _permit = service
        .operation
        .clone()
        .try_lock_owned()
        .map_err(|_| ErrorResponse::conflict("An OpenClaw operation is already running"))?;
    if !service.installed() {
        return Err(ErrorResponse::bad_request("Install OpenClaw first"));
    }
    if service.running().await && !service.managed().await {
        return Err(ErrorResponse::conflict("This gateway was started outside HeraMind. Stop its launcher first, then configure it here."));
    }
    let manager = heramind_agent::llm_backends::get_instance_manager()
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;
    let mut settings = service.settings().map_err(failure)?;
    let mut credentials = service.credentials().map_err(failure)?;
    if let Some(key) = request.api_key.filter(|key| !key.trim().is_empty()) {
        if key.len() > 4096 || !key.is_ascii() || key.bytes().any(|b| b.is_ascii_control()) {
            return Err(ErrorResponse::bad_request("Invalid AgentRouter API key"));
        }
        credentials.api_key = key.trim().into();
    }
    if credentials.api_key.is_empty() {
        return Err(ErrorResponse::bad_request("Enter the AgentRouter API key"));
    }
    if credentials.gateway_token.is_empty() {
        credentials.gateway_token =
            uuid::Uuid::new_v4().to_string() + &uuid::Uuid::new_v4().to_string();
    }
    let id = request
        .backend_id
        .or(settings.backend_id.clone())
        .unwrap_or_else(|| heramind_storage::LlmBackendStore::generate_id("openai"));
    let mut instance = manager.get_instance(&id).unwrap_or_else(|| {
        heramind_storage::LlmBackendInstance::new(
            id.clone(),
            request.name.clone(),
            heramind_storage::LlmBackendType::OpenAi,
        )
    });
    instance.name = request.name.trim().into();
    instance.backend_type = heramind_storage::LlmBackendType::OpenAi;
    instance.endpoint = Some(ENDPOINT.into());
    instance.model = "openclaw/default".into();
    instance.api_key = Some(credentials.gateway_token.clone());
    instance.updated_at = chrono::Utc::now().timestamp();
    instance.validate().map_err(ErrorResponse::bad_request)?;
    let active = instance.is_active;
    settings.model = request.model.trim().into();
    settings.protocol = request.protocol;
    settings.backend_id = Some(id.clone());
    settings.enabled = true;
    service.stop_child().await.map_err(failure)?;
    private_write(
        &service.root.join("secrets.json"),
        &serde_json::to_vec(&credentials).map_err(|e| ErrorResponse::internal(e.to_string()))?,
    )
    .map_err(failure)?;
    private_write(
        &service.root.join("service.json"),
        &serde_json::to_vec(&settings).unwrap(),
    )
    .map_err(failure)?;
    service.start().await.map_err(failure)?;
    manager
        .upsert_instance(instance)
        .await
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;
    if active {
        let _ = crate::handlers::llm_backends::activate_backend_handler(
            State(state),
            axum::extract::Path(id.clone()),
        )
        .await?;
    }
    ok(json!({"backend_id": id}))
}
#[utoipa::path(post, path = "/api/agentrouter-service/start", tag = "llm-backends", responses((status = 200, description = "Start OpenClaw and enable startup with HeraMind")))]
pub async fn start_handler(Extension(admin): Extension<SessionInfo>) -> HandlerResult<Value> {
    require_admin(&admin)?;
    let service = service();
    let _permit = service
        .operation
        .clone()
        .try_lock_owned()
        .map_err(|_| ErrorResponse::conflict("An OpenClaw operation is already running"))?;
    service.start().await.map_err(failure)?;
    let mut settings = service.settings().map_err(failure)?;
    settings.enabled = true;
    private_write(
        &service.root.join("service.json"),
        &serde_json::to_vec(&settings).unwrap(),
    )
    .map_err(failure)?;
    ok(json!({"started": true}))
}
#[utoipa::path(post, path = "/api/agentrouter-service/stop", tag = "llm-backends", responses((status = 200, description = "Stop managed OpenClaw and disable automatic startup")))]
pub async fn stop_handler(Extension(admin): Extension<SessionInfo>) -> HandlerResult<Value> {
    require_admin(&admin)?;
    let service = service();
    let _permit = service
        .operation
        .clone()
        .try_lock_owned()
        .map_err(|_| ErrorResponse::conflict("An OpenClaw operation is already running"))?;
    if service.running().await && !service.managed().await {
        return Err(ErrorResponse::conflict(
            "Stop the external OpenClaw launcher in its terminal first",
        ));
    }
    let mut settings = service.settings().map_err(failure)?;
    settings.enabled = false;
    private_write(
        &service.root.join("service.json"),
        &serde_json::to_vec(&settings).unwrap(),
    )
    .map_err(failure)?;
    service.stop_child().await.map_err(failure)?;
    service.phase("stopped").await;
    ok(json!({"stopped": true}))
}

/// Opt-in supervision; fresh installations do not install or start anything.
pub fn start_supervisor() {
    let service = service();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            if service.settings().is_ok_and(|s| s.enabled)
                && service.installed()
                && !service.running().await
            {
                if let Ok(_permit) = service.operation.try_lock() {
                    if let Err(error) = service.start().await {
                        service.progress.lock().await.error = Some(error.to_string());
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_uses_secret_refs_and_the_requested_protocol() {
        let config = make_config("gpt-6.1-sol", "openai-responses").unwrap();
        assert_eq!(
            config["models"]["providers"]["agentrouter"]["api"],
            "openai-responses"
        );
        assert_eq!(config["gateway"]["bind"], "loopback");
        assert_eq!(config["tools"]["deny"], json!(["*"]));
        assert!(config["models"]["providers"]["agentrouter"]["apiKey"].is_object());
        assert!(make_config("bad\nmodel", "openai-responses").is_err());
        assert!(make_config("model", "shell-command").is_err());
        assert_eq!(
            make_config("claude-test", "anthropic-messages").unwrap()["models"]["providers"]
                ["agentrouter"]["baseUrl"],
            "https://agentrouter.org"
        );
    }
    #[test]
    fn checksum_names_are_exact_and_failed_replacement_keeps_the_runtime() {
        let hash = "a".repeat(64);
        assert_eq!(
            checksum_for(&format!("{hash}  node.tar.gz\n"), "node.tar.gz").unwrap(),
            hash
        );
        assert!(checksum_for(&format!("{hash}  wrong.tar.gz\n"), "node.tar.gz").is_err());
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("runtime");
        std::fs::create_dir(&current).unwrap();
        std::fs::write(current.join("keep"), "ok").unwrap();
        assert!(replace_directory(&dir.path().join("missing"), &current).is_err());
        assert_eq!(std::fs::read_to_string(current.join("keep")).unwrap(), "ok");
    }
    #[test]
    fn status_types_do_not_serialize_credentials_and_defaults_are_opt_in() {
        assert!(!Settings::default().enabled);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.json");
        private_write(
            &path,
            br#"{"apiKey":"test-only","gatewayToken":"local-test"}"#,
        )
        .unwrap();
        assert_eq!(
            read_or_default::<Credentials>(&path).unwrap().api_key,
            "test-only"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let session = SessionInfo {
            user_id: "user".into(),
            username: "user".into(),
            role: UserRole::User,
            created_at: 0,
            expires_at: 1,
        };
        assert_eq!(
            require_admin(&session).unwrap_err().status,
            StatusCode::FORBIDDEN
        );
    }
}
