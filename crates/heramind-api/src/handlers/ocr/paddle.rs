use std::{io::Cursor, path::Path, sync::Arc, time::Duration};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use heramind_core::extension::ExtensionRuntime;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::{
    io::AsyncReadExt,
    sync::{Mutex, Semaphore},
    time::Instant,
};

use super::Language;

const EXTENSION_ID: &str = "paddle-ocr-vl";
const MAX_IMAGE_BYTES: usize = 6 * 1024 * 1024;
const HEALTH_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone, Default, Deserialize)]
pub struct Health {
    pub status: String,
    pub version: Option<String>,
    pub model_loaded: bool,
}

impl Health {
    pub fn available(&self) -> bool {
        self.status == "ok"
    }
}

pub struct Paddle {
    runtime: Arc<ExtensionRuntime>,
    last_health: Mutex<Option<(Instant, Health)>>,
    health_probe: Arc<Semaphore>,
}

impl Paddle {
    pub fn new(runtime: Arc<ExtensionRuntime>) -> Self {
        Self {
            runtime,
            last_health: Mutex::new(None),
            health_probe: Arc::new(Semaphore::new(1)),
        }
    }

    pub async fn health(&self) -> Health {
        let mut last = self.last_health.lock().await;
        if let Some((checked, health)) = last.as_ref() {
            if checked.elapsed() < HEALTH_INTERVAL {
                return health.clone();
            }
        }
        let health = self.probe().await;
        *last = Some((Instant::now(), health.clone()));
        health
    }

    async fn probe(&self) -> Health {
        let Ok(permit) = self.health_probe.clone().try_acquire_owned() else {
            return Health::default();
        };
        let runtime = self.runtime.clone();
        // Keep the runtime's own timeout and IPC cleanup active if the dashboard
        // stops waiting or the client disconnects.
        let probe = tokio::spawn(async move {
            let _permit = permit;
            runtime
                .execute_command(EXTENSION_ID, "health", &json!({}))
                .await
        });
        let result = tokio::time::timeout(Duration::from_secs(7), probe).await;
        match result {
            Ok(Ok(Ok(value))) => serde_json::from_value(value).unwrap_or_default(),
            _ => Health::default(),
        }
    }

    pub async fn recognize(
        &self,
        image: &Path,
        language: Language,
    ) -> Result<String, &'static str> {
        let mut bytes = Vec::new();
        tokio::fs::File::open(image)
            .await
            .map_err(|_| "STORAGE_ERROR")?
            .take((MAX_IMAGE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| "STORAGE_ERROR")?;
        if bytes.len() > MAX_IMAGE_BYTES {
            return Err("IMAGE_TOO_LARGE");
        }
        let (width, height) =
            image::ImageReader::with_format(Cursor::new(&bytes), image::ImageFormat::Png)
                .into_dimensions()
                .map_err(|_| "ENGINE_ERROR")?;
        let language = match language {
            Language::Eng => "en",
            Language::Vie | Language::VieEng => "multilingual",
        };
        let args = json!({
            "image_base64": STANDARD.encode(bytes),
            "image_width": width,
            "image_height": height,
            "language": language,
        });
        let mut result = self
            .runtime
            .execute_command(EXTENSION_ID, "recognize", &args)
            .await
            .map_err(|_| "PADDLE_INFERENCE_ERROR")?;
        let Some(Value::String(text)) = result.get_mut("full_text").map(Value::take) else {
            return Err("PADDLE_INFERENCE_ERROR");
        };
        Ok(text)
    }
}
