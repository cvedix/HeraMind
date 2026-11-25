use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use tokio::{io::AsyncReadExt, process::Command, time::timeout};

use super::{Language, MAX_PAGES};

pub const MAX_PAGE_TEXT: usize = 128 * 1024;

#[derive(Clone)]
pub struct Engine {
    bin_dir: Option<PathBuf>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            bin_dir: std::env::var_os("HERAMIND_OCR_BIN_DIR").map(PathBuf::from),
        }
    }

    fn command(&self, executable: &str) -> Command {
        let program = self
            .bin_dir
            .as_ref()
            .map_or_else(|| PathBuf::from(executable), |dir| dir.join(executable));
        let mut command = Command::new(program);
        command.env("LC_ALL", "C").env("OMP_THREAD_LIMIT", "1");
        command
    }

    pub async fn check_pdf_tools(&self) -> Result<(), &'static str> {
        self.run(self.command("pdfinfo").arg("-v"), 4096).await?;
        self.run(self.command("pdftoppm").arg("-v"), 4096).await?;
        Ok(())
    }

    pub async fn languages(&self) -> Result<Vec<Language>, &'static str> {
        let output = self
            .run(self.command("tesseract").arg("--list-langs"), 8192)
            .await?;
        let has = |lang| output.lines().any(|line| line.trim() == lang);
        Ok(Language::ALL
            .into_iter()
            .filter(|language| match language {
                Language::Vie => has("vie"),
                Language::Eng => has("eng"),
                Language::VieEng => has("vie") && has("eng"),
            })
            .collect())
    }

    async fn run(&self, command: &mut Command, max_output: usize) -> Result<String, &'static str> {
        command
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .kill_on_drop(true);
        timeout(Duration::from_secs(30), async {
            let mut child = command.spawn().map_err(|_| "ENGINE_UNAVAILABLE")?;
            let mut data = Vec::new();
            child
                .stdout
                .take()
                .ok_or("ENGINE_UNAVAILABLE")?
                .take((max_output + 1) as u64)
                .read_to_end(&mut data)
                .await
                .map_err(|_| "ENGINE_ERROR")?;
            if data.len() > max_output {
                return Err("RESULT_TOO_LARGE");
            }
            if !child.wait().await.map_err(|_| "ENGINE_ERROR")?.success() {
                return Err("ENGINE_ERROR");
            }
            String::from_utf8(data).map_err(|_| "ENGINE_ERROR")
        })
        .await
        .map_err(|_| "STAGE_TIMEOUT")?
    }

    pub async fn page_count(&self, pdf: &Path) -> Result<u32, &'static str> {
        let info = self
            .run(self.command("pdfinfo").arg(pdf), 64 * 1024)
            .await
            .map_err(|error| {
                if error == "ENGINE_ERROR" {
                    "UNREADABLE_PDF"
                } else {
                    error
                }
            })?;
        let count = info
            .lines()
            .find_map(|line| line.strip_prefix("Pages:"))
            .and_then(|value| value.trim().parse::<u32>().ok())
            .ok_or("UNREADABLE_PDF")?;
        if count == 0 {
            Err("UNREADABLE_PDF")
        } else if count > MAX_PAGES {
            Err("TOO_MANY_PAGES")
        } else {
            Ok(count)
        }
    }

    pub async fn render(
        &self,
        pdf: &Path,
        scratch: &Path,
        number: u32,
    ) -> Result<PathBuf, &'static str> {
        let prefix = scratch.join("page");
        let page = number.to_string();
        let args = [
            "-f",
            &page,
            "-l",
            &page,
            "-scale-to",
            "2400",
            "-gray",
            "-singlefile",
            "-png",
        ];
        self.run(
            self.command("pdftoppm").args(args).arg(pdf).arg(&prefix),
            4096,
        )
        .await?;
        Ok(prefix.with_extension("png"))
    }

    pub async fn recognize(
        &self,
        image: &Path,
        language: Language,
    ) -> Result<String, &'static str> {
        self.run(
            self.command("tesseract").arg(image).args([
                OsString::from("stdout"),
                "-l".into(),
                language.engine_value().into(),
                "--psm".into(),
                "3".into(),
            ]),
            MAX_PAGE_TEXT,
        )
        .await
    }
}
