use crate::config::models::ConfigOpts;
use serde::Serialize;
use std::{fmt::Debug, fmt::Display};

fn __display<T>(opt: Option<&T>) -> String
where
    T: Serialize + Debug,
{
    if let Some(opt) = opt {
        match ron::to_string(opt) {
            Ok(ron) => ron,
            Err(_) => format!("{:?}", opt),
        }
    } else {
        "None".to_string()
    }
}

impl Display for ConfigOpts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = try {
            // Verify
            writeln!(
                f,
                "Token: {}",
                __display(self.verify.as_ref().and_then(|v| v.token.as_ref()))
            )?;

            // List
            writeln!(
                f,
                "Include zones: {}",
                __display(
                    self.list.as_ref().and_then(|l| l.include_zones.as_ref())
                )
            )?;
            writeln!(
                f,
                "Ignore zones: {}",
                __display(
                    self.list.as_ref().and_then(|l| l.ignore_zones.as_ref())
                )
            )?;
            writeln!(
                f,
                "Include records: {}",
                __display(
                    self.list.as_ref().and_then(|l| l.include_records.as_ref())
                )
            )?;
            writeln!(
                f,
                "Ignore records: {}",
                __display(
                    self.list.as_ref().and_then(|l| l.ignore_records.as_ref())
                )
            )?;

            // Inventory
            writeln!(
                f,
                "Inventory path: {}",
                __display(
                    self.inventory.as_ref().and_then(|i| i.path.as_ref())
                )
            )?;
            writeln!(
                f,
                "Commit without user prompt (force): {}",
                __display(self.commit.as_ref().map(|c| &c.force))
            )?;
            writeln!(
                f,
                "Watch interval: {}",
                __display(
                    self.watch.as_ref().and_then(|w| w.interval.as_ref())
                )
            )?;
        };
        result
    }
}
