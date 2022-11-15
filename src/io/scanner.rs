use anyhow::Result;
use std::{fmt::Display, io::Write, str::FromStr};
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
        std::io::stdout().write_all(format!("{}: > ", prompt).as_bytes())?;
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

    /// Prompt the user for a yes (true) or no (false).
    pub async fn prompt_yes_or_no(
        &mut self,
        prompt: impl Display,
    ) -> Result<Option<bool>> {
        let prompt_str = prompt.to_string();
        let answer = 'control: loop {
            match Self::prompt(self, &prompt_str).await? {
                Some(input) => match input.to_lowercase().as_str() {
                    "y" | "yes" => break Some(true),
                    "n" | "no" => break Some(false),
                    _ => continue 'control,
                },
                None => break None,
            }
        };
        Ok(answer)
    }

    /// Prompt the user for a type and collect it.
    pub async fn prompt_t<T>(
        &mut self,
        prompt: impl Display,
    ) -> Result<Option<T>>
    where
        T: FromStr,
    {
        let path = loop {
            match self.prompt(&prompt).await? {
                Some(input) => match input.parse::<T>() {
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
    }
}
