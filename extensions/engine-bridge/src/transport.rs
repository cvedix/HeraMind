use crate::{failure, MAX_REQUEST, MAX_RESPONSE};
use heramind_extension_sdk::Result;
use serde_json::{json, Value};
use std::os::unix::fs::PermissionsExt;
use std::{path::Path, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};

// Each runner owns a persistent child. No listening socket or public engine endpoint.
pub(crate) struct Worker {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    pub(crate) synchronized: bool,
}

impl Worker {
    pub(crate) fn start(program: &Path, root: &Path) -> Result<Self> {
        // HeraMind's package extractor writes assets as regular files and does
        // not restore ZIP executable bits. Repair only our fixed worker asset.
        let metadata = std::fs::metadata(program).map_err(failure)?;
        if !metadata.is_file() {
            return Err(failure("Engine worker is not a regular file"));
        }
        let mode = metadata.permissions().mode();
        if mode & 0o100 == 0 {
            std::fs::set_permissions(
                program,
                std::fs::Permissions::from_mode((mode & 0o777) | 0o100),
            )
            .map_err(failure)?;
        }
        Self::spawn(Command::new(program), root)
    }

    #[cfg(test)]
    pub(crate) fn test_peer(script: &Path, root: &Path) -> Result<Self> {
        // Execute an installed interpreter rather than execve a file that a
        // parallel test has just created (ETXTBSY on some filesystems).
        let mut command = Command::new("/usr/bin/python3");
        command.arg(script);
        Self::spawn(command, root)
    }

    fn spawn(mut command: Command, root: &Path) -> Result<Self> {
        // RUNPATH on the executable does not cover every transitive dependency.
        // Prefer the libraries packaged alongside this worker over host copies.
        let mut libraries = vec![root.join("assets/lib")];
        if let Some(existing) = std::env::var_os("LD_LIBRARY_PATH") {
            libraries.extend(std::env::split_paths(&existing));
        }
        command.env(
            "LD_LIBRARY_PATH",
            std::env::join_paths(libraries).map_err(failure)?,
        );
        let mut child = command
            .current_dir(root)
            // Bound decoder allocations before OpenCV initializes its image codecs.
            .env("OPENCV_IO_MAX_IMAGE_PIXELS", "16777216")
            .env("OPENCV_IO_MAX_IMAGE_WIDTH", "8192")
            .env("OPENCV_IO_MAX_IMAGE_HEIGHT", "8192")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| failure(format!("Cannot start engine worker: {e}")))?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| failure("Worker stdin missing"))?;
        let output = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| failure("Worker stdout missing"))?,
        );
        Ok(Self {
            child,
            input,
            output,
            synchronized: true,
        })
    }

    pub(crate) async fn exchange(&mut self, request: &Value, timeout_ms: u64) -> Result<Value> {
        let encoded = serde_json::to_vec(request).map_err(failure)?;
        if encoded.len() > MAX_REQUEST {
            return Err(failure("Engine request exceeds 8 MiB"));
        }
        // A cancelled future must leave the stream poisoned. The next caller replaces
        // the child instead of consuming an old response as the next request's result.
        self.synchronized = false;
        let response = tokio::time::timeout(Duration::from_millis(timeout_ms), async {
            self.input
                .write_u32(encoded.len() as u32)
                .await
                .map_err(failure)?;
            self.input.write_all(&encoded).await.map_err(failure)?;
            self.input.flush().await.map_err(failure)?;
            let length = self.output.read_u32().await.map_err(failure)? as usize;
            if length == 0 || length > MAX_RESPONSE {
                return Err(failure("Invalid engine response length"));
            }
            let mut bytes = vec![0; length];
            self.output.read_exact(&mut bytes).await.map_err(failure)?;
            let response: Value = serde_json::from_slice(&bytes).map_err(failure)?;
            if response.get("protocol") != Some(&json!(1))
                || response.get("request_id") != request.get("request_id")
                || !response["ok"].is_boolean()
                || (response["ok"] == true && response.get("data").is_none())
            {
                return Err(failure("Engine protocol or request ID mismatch"));
            }
            Ok(response)
        })
        .await
        .map_err(|_| failure("Engine timed out; worker will be restarted"))??;
        self.synchronized = true;
        Ok(response)
    }

    pub(crate) async fn shutdown(&mut self) {
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
    }
}
