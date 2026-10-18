use std::path::PathBuf;

use clap::{ArgGroup, Parser};

use crate::i18n::{t_args, LangArg, Msg};

#[derive(Parser, Debug)]
#[command(
    name = "another-vantage",
    version,
    about = "Lenovo Vantage features for Linux (read-only CLI first)"
)]
#[command(group(
    ArgGroup::new("action")
        .required(true)
        .multiple(false)
        .args(["info", "config", "firmware"])
))]
pub struct Cli {
    /// Read basic information (all/model/spec)
    #[arg(long, value_name = "ITEM", num_args = 0..=1, default_missing_value = "all")]
    pub info: Option<String>,

    /// Read hardware configuration state (all/performance/charge/lid/power/io)
    #[arg(long, value_name = "ITEM", num_args = 0..=1, default_missing_value = "all")]
    pub config: Option<String>,

    /// Check BIOS version and firmware update availability
    #[arg(long)]
    pub firmware: bool,

    /// Machine-readable JSON output
    #[arg(long)]
    pub json: bool,

    /// Disable colors
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Verbose diagnostics
    #[arg(long)]
    pub verbose: bool,

    /// Output language (auto follows LANG/LC_ALL)
    #[arg(long, value_enum, default_value_t = LangArg::Auto)]
    pub lang: LangArg,

    /// sysfs root for testing (hidden option)
    #[arg(long, hide = true, default_value = "/")]
    pub sysfs_root: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Info,
    Config,
    Firmware,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    All,
    Model,
    Spec,
    Performance,
    Charge,
    Lid,
    Power,
    Io,
}

impl Cli {
    pub fn group(&self) -> Group {
        if self.info.is_some() {
            Group::Info
        } else if self.config.is_some() {
            Group::Config
        } else {
            Group::Firmware
        }
    }

    pub fn raw_item(&self) -> &str {
        match self.group() {
            Group::Info => self.info.as_deref().unwrap_or("all"),
            Group::Config => self.config.as_deref().unwrap_or("all"),
            Group::Firmware => "firmware",
        }
    }
}

pub fn parse_item(group: Group, raw: &str) -> Result<Item, String> {
    let s = raw.to_ascii_lowercase();
    let item = match (group, s.as_str()) {
        (_, "all") => Item::All,
        (Group::Info, "model") => Item::Model,
        (Group::Info, "spec") => Item::Spec,
        (Group::Config, "performance") => Item::Performance,
        (Group::Config, "charge") => Item::Charge,
        (Group::Config, "lid") => Item::Lid,
        (Group::Config, "power") => Item::Power,
        (Group::Config, "io") => Item::Io,
        (Group::Info, other) => return Err(t_args(Msg::UnknownInfoItem, &[("other", other)])),
        (Group::Config, other) => return Err(t_args(Msg::UnknownConfigItem, &[("other", other)])),
        (Group::Firmware, _) => Item::All,
    };
    Ok(item)
}
