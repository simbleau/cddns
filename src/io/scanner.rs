use anyhow::Result;
use std::{
    ffi::OsString,
    fmt::Display,
    io::Write,
    path::{Path, PathBuf},
    thread::JoinHandle,
};
use tokio::runtime::Handle;

/// A stdin scanner to collect user input on command line.
pub struct Scanner {
    rx: tokio::sync::mpsc::Receiver<String>,
}

impl Scanner {
    /// Create a new scanner.
    pub fn new(runtime: Handle) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let mut line_buf = String::new();
            while stdin.read_line(&mut line_buf).is_ok() {
                let line = line_buf.trim().to_string();
                line_buf.clear();
                {
                    let tx = tx.clone();
                    runtime.spawn(async move { tx.send(line).await });
                }
            }
        });
        Self { rx }
    }

    /// Prompt the user for an answer and collect it.
    pub async fn prompt(&mut self, prompt: impl Display) -> Result<String> {
        std::io::stdout().write(format!("{}: > ", prompt).as_bytes())?;
        std::io::stdout().flush()?;

        tokio::select! {
            Some(line) = self.rx.recv() => {
                match line.to_lowercase().trim() {
                    "exit" | "quit" => {
                        anyhow::bail!("aborted")
                    },
                    _ => {
                        Ok(line)
                    }
                }
            }
        }
    }

    // Prompt the user for a path and collect it.
    pub async fn prompt_path<P>(&mut self, default_path: P) -> Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        let name = default_path
            .as_ref()
            .file_name()
            .expect("invalid default path file name");
        let ext = default_path
            .as_ref()
            .extension()
            .expect("invalid default file extension");
        // Get output path
        let mut output_path = OsString::from(
            self.prompt(format!(
                "Output path [default: {}]",
                default_path.as_ref().display()
            ))
            .await?,
        );
        if output_path.is_empty() {
            output_path.push(name);
        }
        let output_path = PathBuf::from(&output_path).with_extension(ext);
        if output_path.exists() {
            let overwrite = loop {
                match self
                    .prompt("File location exists, overwrite? (y/N)")
                    .await?
                    .to_lowercase()
                    .trim()
                {
                    "y" | "yes" => break true,
                    "" | "n" | "no" | "exit" | "quit" => break false,
                    _ => continue,
                }
            };

            if overwrite {
                tokio::fs::remove_file(&output_path).await?;
            } else {
                anyhow::bail!("aborted")
            }
        }
        Ok(output_path)
    }
}

impl Drop for Scanner {
    /// Close communication and drop the scanner, which may result in lost
    /// messages.
    fn drop(&mut self) {
        self.rx.close();
        drop(self);
    }
}
