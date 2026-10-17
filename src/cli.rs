use std::path::PathBuf;

use clap::{ArgGroup, Parser};

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
    /// 读取基本信息 (all/model/spec)
    #[arg(long, value_name = "ITEM", num_args = 0..=1, default_missing_value = "all")]
    pub info: Option<String>,

    /// 读取硬件配置状态 (all/performance/charge/lid/power/io)
    #[arg(long, value_name = "ITEM", num_args = 0..=1, default_missing_value = "all")]
    pub config: Option<String>,

    /// 检查 BIOS 版本与固件更新可用性
    #[arg(long)]
    pub firmware: bool,

    /// 机器可读 JSON 输出
    #[arg(long)]
    pub json: bool,

    /// 禁用颜色
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// 详细日志
    #[arg(long)]
    pub verbose: bool,

    /// 用于测试的 sysfs 根目录（隐藏选项）
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
        (Group::Info, other) => {
            return Err(format!(
                "未知的 --info 子项 “{other}”；可用: all, model, spec（performance/charge/lid/power/io 属于 --config）"
            ))
        }
        (Group::Config, other) => {
            return Err(format!(
                "未知的 --config 子项 “{other}”；可用: all, performance, charge, lid, power, io（model/spec 属于 --info）"
            ))
        }
        (Group::Firmware, _) => Item::All,
    };
    Ok(item)
}
