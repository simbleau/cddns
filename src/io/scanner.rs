use anyhow::Result;
use std::{fmt::Display, io::Write, path::PathBuf};
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
    pub async fn prompt(
        &mut self,
        prompt: impl Display,
    ) -> Result<Option<String>> {
        std::io::stdout().write(format!("{}: > ", prompt).as_bytes())?;
        std::io::stdout().flush()?;

        tokio::select! {
                Some(line) = self.rx.recv() => {
                    match line.to_lowercase().trim() {
                        "exit" | "quit" => {
                            anyhow::bail!("aborted")
                        },
                        "" => {
                            Ok(None)
                        }
                        _ => {
                            Ok(Some(line.trim().to_owned()))
                        }
                    }
                }
        }
    }

    /// Prompt the user for a path and collect it.
    pub async fn prompt_path(
        &mut self,
        prompt: impl Display,
    ) -> Result<Option<PathBuf>> {
        let path = loop {
            match self.prompt(&prompt).await? {
                Some(input) => match input.parse::<PathBuf>() {
                    Ok(pb) => break Some(pb),
                    _ => continue,
                },
                None => break None,
            }
        };
        Ok(path)
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
