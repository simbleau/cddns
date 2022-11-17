use crate::config::models::ConfigOpts;
use serde::Serialize;
use std::{fmt::Debug, fmt::Display};

fn encode_opt<T>(opt: Option<&T>) -> String
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
        write!(
            f,
            r#"Token: {}
Include zones: {}
Ignore zones: {}
Include records: {}
Ignore records: {}
Inventory path: {}
Force commit: {}
Watch interval: {}ms"#,
            encode_opt(self.verify.as_ref().and_then(|v| v.token.as_ref())),
            encode_opt(
                self.list.as_ref().and_then(|l| l.include_zones.as_ref())
            ),
            encode_opt(
                self.list.as_ref().and_then(|l| l.ignore_zones.as_ref())
            ),
            encode_opt(
                self.list.as_ref().and_then(|l| l.include_records.as_ref())
            ),
            encode_opt(
                self.list.as_ref().and_then(|l| l.ignore_records.as_ref())
            ),
            encode_opt(self.inventory.as_ref().and_then(|i| i.path.as_ref())),
            encode_opt(self.commit.as_ref().map(|c| &c.force)),
            encode_opt(self.watch.as_ref().and_then(|w| w.interval.as_ref()))
        )
    }
}
