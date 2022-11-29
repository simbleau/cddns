use anyhow::{bail, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use serde::de::DeserializeOwned;
use std::{fmt::Display, io::Write, str::FromStr};

/// A stdin scanner to collect user input on command line.
pub struct Scanner;

impl Scanner {
    fn display(prompt: impl Display, type_hint: impl Display) -> Result<()> {
        std::io::stdout()
            .write_all(format!("{} ~ ({}) > ", prompt, type_hint).as_bytes())?;
        Ok(std::io::stdout().flush()?)
    }

    /// Read a line from stdin (blocking).
    pub fn read_line() -> Result<Option<String>> {
        let mut line = String::new();
        while let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Char(c) => {
                    line.push(c);
                }
                _ => {}
            }
        }
        if line.is_empty() {
            Ok(None)
        } else {
            Ok(Some(line))
        }
    }
}

/// Prompt the user for an answer and collect it.
pub fn prompt(
    prompt: impl Display,
    type_hint: impl Display,
) -> Result<Option<String>> {
    let prompt = prompt.to_string();
    let type_hint = type_hint.to_string();
    loop {
        Scanner::display(&prompt, &type_hint)?;
        let line = Scanner::read_line()?;
        if let Some(line) = line {
            match line.to_lowercase().trim() {
                "exit" | "quit" => {
                    bail!("aborted")
                }
                _ => break Ok(Some(line.trim().to_owned())),
            }
        } else if let None = line {
            break Ok(None);
        }
    }
}

/// Prompt the user for a yes (true) or no (false).
pub fn prompt_yes_or_no(
    prompt: impl Display,
    type_hint: impl Display,
) -> Result<Option<bool>> {
    let prompt = prompt.to_string();
    let type_hint = type_hint.to_string();
    loop {
        Scanner::display(&prompt, &type_hint)?;
        let line = Scanner::read_line()?;
        if let Some(input) = line {
            match input.to_lowercase().as_str() {
                "y" | "yes" => break Ok(Some(true)),
                "n" | "no" => break Ok(Some(false)),
                _ => {
                    println!(
                            "Error parsing input. Expected 'yes' or 'no'. Try again."
                        );
                    continue;
                }
            }
        } else if let None = line {
            break Ok(None);
        }
    }
}

/// Prompt the user for a type and collect it.
pub fn prompt_t<T>(
    prompt: impl Display,
    type_hint: impl Display,
) -> Result<Option<T>>
where
    T: FromStr,
{
    let prompt = prompt.to_string();
    let type_hint = type_hint.to_string();
    loop {
        match crate::io::scanner::prompt(&prompt, &type_hint)? {
            Some(input) => match input.parse::<T>() {
                Ok(pb) => break Ok(Some(pb)),
                _ => {
                    println!(
                        "Error parsing input. Expected {}. Try again.",
                        &type_hint
                    );
                    continue;
                }
            },
            None => break Ok(None),
        }
    }
}

/// Prompt the user for a type in RON notation (https://github.com/ron-rs/ron).
pub fn prompt_ron<T>(
    prompt: impl Display,
    type_hint: impl Display,
) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let prompt = prompt.to_string();
    let type_hint = type_hint.to_string();
    let ron = loop {
        match crate::io::scanner::prompt(&prompt, &type_hint)? {
            Some(input) => match ron::from_str(&input) {
                Ok(pb) => break Some(pb),
                _ => {
                    println!("Error parsing input. Input should be in RON notation (https://github.com/ron-rs/ron/wiki/Specification)");
                    continue;
                }
            },
            None => break None,
        }
    };
    Ok(ron)
}
