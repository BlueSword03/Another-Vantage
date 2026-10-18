//! Tiny dependency-free message catalog. Two languages, resolved once at
//! startup from `--lang` or the locale environment.

use std::sync::atomic::{AtomicU8, Ordering};

use clap::{CommandFactory, ValueEnum};

use crate::cli::Cli;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum LangArg {
    /// Follow LC_ALL / LC_MESSAGES / LANG
    Auto,
    /// 中文
    Zh,
    /// English
    En,
}

impl std::fmt::Display for LangArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LangArg::Auto => "auto",
            LangArg::Zh => "zh",
            LangArg::En => "en",
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn lang_of_locale(locale: Option<&str>) -> Lang {
    match locale {
        Some(l) if l.to_ascii_lowercase().starts_with("zh") => Lang::Zh,
        _ => Lang::En,
    }
}

fn locale() -> Option<String> {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(key) {
            if !v.is_empty() && v != "C" && v != "POSIX" {
                return Some(v);
            }
        }
    }
    None
}

pub fn resolve(arg: LangArg) -> Lang {
    match arg {
        LangArg::Zh => Lang::Zh,
        LangArg::En => Lang::En,
        LangArg::Auto => lang_of_locale(locale().as_deref()),
    }
}

/// Pre-scan argv so `--help` is localized before clap parses the arguments.
pub fn pre_scan(args: &[String]) -> LangArg {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if let Some(v) = a.strip_prefix("--lang=") {
            if let Ok(arg) = <LangArg as ValueEnum>::from_str(v, true) {
                return arg;
            }
        } else if a == "--lang" {
            if let Some(v) = it.next() {
                if let Ok(arg) = <LangArg as ValueEnum>::from_str(v, true) {
                    return arg;
                }
            }
        }
    }
    LangArg::Auto
}

pub fn init(lang: Lang) {
    CURRENT.store(
        match lang {
            Lang::Zh => 1,
            Lang::En => 0,
        },
        Ordering::Relaxed,
    );
}

pub fn current() -> Lang {
    if CURRENT.load(Ordering::Relaxed) == 1 {
        Lang::Zh
    } else {
        Lang::En
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Msg {
    About,
    AfterHelp,
    HelpInfo,
    HelpConfig,
    HelpFirmware,
    HelpJson,
    HelpNoColor,
    HelpVerbose,
    HelpSysfsRoot,
    HelpLang,
    Unknown,
    Unavailable,
    NodeMissing,
    NeedRoot,
    UseRoot,
    NotNumber,
    NotBool,
    FirmwareNote,
    BiosEnhancedNote,
    PpdNote,
    PpdErr,
    NoBattery,
    ChargeThreshold,
    ChargeNote,
    UsbThreshold,
    UsbNote,
    LidFlipMissing,
    LidInstantMissing,
    LidNote,
    NoVpc,
    IoNote,
    DmiSpeed,
    ErrorPrefix,
    UnknownInfoItem,
    UnknownConfigItem,
}

impl Msg {
    fn zh(self) -> &'static str {
        match self {
            Msg::About => {
                "适用于 Linux 的联想 Vantage 功能（只读 CLI：不写入设置、不做参数监控）"
            }
            Msg::AfterHelp => {
                "只读快照：读取并打印当前硬件与配置状态，不修改任何设置，也不持续监控参数。\n写入与监视功能在后续阶段实现。"
            }
            Msg::HelpInfo => "读取基本信息 (all/model/spec)",
            Msg::HelpConfig => "读取硬件配置状态 (all/performance/charge/lid/power/io)",
            Msg::HelpFirmware => "检查 BIOS 版本与固件更新可用性",
            Msg::HelpJson => "机器可读 JSON 输出",
            Msg::HelpNoColor => "禁用颜色",
            Msg::HelpVerbose => "详细日志",
            Msg::HelpSysfsRoot => "用于测试的 sysfs 根目录（隐藏选项）",
            Msg::HelpLang => "输出语言（auto 按 LANG/LC_ALL 判断，默认 auto）",
            Msg::Unknown => "未知",
            Msg::Unavailable => "不可用",
            Msg::NodeMissing => "节点不存在",
            Msg::NeedRoot => "需要管理员权限",
            Msg::UseRoot => "请使用 root 或 sudo 执行",
            Msg::NotNumber => "无法解析为数字",
            Msg::NotBool => "无法解析为布尔值",
            Msg::FirmwareNote => {
                "运行 `fwupdmgr get-updates` 查看可用更新；21CY 的 BIOS1/System Firmware 标记为“固件更新不可用”，无 LVFS 刷写路径"
            }
            Msg::BiosEnhancedNote => {
                "21CY 的 thinklmi 未导出性能增强属性；请在 BIOS 中查看，Linux 侧不可读写"
            }
            Msg::PpdNote => {
                "PPD active 时优先通过 power-profiles-daemon 设置，避免直接写 sysfs 造成状态不一致"
            }
            Msg::PpdErr => "无法检测 power-profiles-daemon: {e}",
            Msg::NoBattery => "未发现 BAT* 设备",
            Msg::ChargeThreshold => "无 charge_control_*_threshold；21CY 不支持充电阈值",
            Msg::ChargeNote => "conservation_mode 已 deprecated；写入应使用 charge_types（需 root）",
            Msg::UsbThreshold => "本机无阈值节点；仅支持开关",
            Msg::UsbNote => {
                "AlwaysOnUSB（thinklmi）与 usb_charging 可能控制同一硬件；写入时只能选一条路径"
            }
            Msg::LidFlipMissing => "本机无 flip_to_start 节点",
            Msg::LidInstantMissing => "本机无 instant_boot 节点",
            Msg::LidNote => "开盖开机是机型相关能力；21CY 不支持",
            Msg::NoVpc => "未找到 VPC2004 设备",
            Msg::IoNote => {
                "current_value 为 0600，读值/写值都需要 root；不要写 SecureBoot/DeviceGuard 等非 IO 项"
            }
            Msg::DmiSpeed => "需要 dmidecode；本机无免 root 通路",
            Msg::ErrorPrefix => "错误：{e}",
            Msg::UnknownInfoItem => {
                "未知的 --info 子项 “{other}”；可用: all, model, spec（performance/charge/lid/power/io 属于 --config）"
            }
            Msg::UnknownConfigItem => {
                "未知的 --config 子项 “{other}”；可用: all, performance, charge, lid, power, io（model/spec 属于 --info）"
            }
        }
    }

    fn en(self) -> &'static str {
        match self {
            Msg::About => {
                "Lenovo Vantage features for Linux (read-only CLI: no writes, no monitoring)"
            }
            Msg::AfterHelp => {
                "Read-only snapshot: prints current hardware and configuration state. It never changes settings and does not continuously monitor values.\nWrites and monitoring are planned for later phases."
            }
            Msg::HelpInfo => "Read basic information (all/model/spec)",
            Msg::HelpConfig => "Read hardware configuration state (all/performance/charge/lid/power/io)",
            Msg::HelpFirmware => "Check BIOS version and firmware update availability",
            Msg::HelpJson => "Machine-readable JSON output",
            Msg::HelpNoColor => "Disable colors",
            Msg::HelpVerbose => "Verbose diagnostics",
            Msg::HelpSysfsRoot => "sysfs root for testing (hidden option)",
            Msg::HelpLang => "Output language (auto follows LANG/LC_ALL; default auto)",
            Msg::Unknown => "unknown",
            Msg::Unavailable => "n/a",
            Msg::NodeMissing => "node not present",
            Msg::NeedRoot => "requires administrator privileges",
            Msg::UseRoot => "run as root (sudo)",
            Msg::NotNumber => "not a number",
            Msg::NotBool => "not a boolean",
            Msg::FirmwareNote => {
                "Run `fwupdmgr get-updates` for available updates; on 21CY, BIOS1/System Firmware are marked \"firmware update unavailable\" with no LVFS path"
            }
            Msg::BiosEnhancedNote => {
                "21CY's thinklmi does not expose the enhanced-performance attribute; check it in BIOS — it is not readable or writable from Linux"
            }
            Msg::PpdNote => {
                "When PPD is active, prefer power-profiles-daemon so direct sysfs writes cannot desync the state"
            }
            Msg::PpdErr => "cannot detect power-profiles-daemon: {e}",
            Msg::NoBattery => "no BAT* device found",
            Msg::ChargeThreshold => "no charge_control_*_threshold; 21CY has no charge threshold",
            Msg::ChargeNote => "conservation_mode is deprecated; write charge_types instead (requires root)",
            Msg::UsbThreshold => "no threshold node on this machine; on/off only",
            Msg::UsbNote => {
                "AlwaysOnUSB (thinklmi) and usb_charging may drive the same hardware; pick one write path"
            }
            Msg::LidFlipMissing => "no flip_to_start node on this machine",
            Msg::LidInstantMissing => "no instant_boot node on this machine",
            Msg::LidNote => "lid-open power-on is model-specific; unsupported on 21CY",
            Msg::NoVpc => "VPC2004 device not found",
            Msg::IoNote => {
                "current_value is 0600, so reads and writes need root; never write non-IO entries such as SecureBoot/DeviceGuard"
            }
            Msg::DmiSpeed => "requires dmidecode; no root-free path on this machine",
            Msg::ErrorPrefix => "error: {e}",
            Msg::UnknownInfoItem => {
                "unknown --info item \"{other}\"; valid: all, model, spec (performance/charge/lid/power/io belong to --config)"
            }
            Msg::UnknownConfigItem => {
                "unknown --config item \"{other}\"; valid: all, performance, charge, lid, power, io (model/spec belong to --info)"
            }
        }
    }

    fn text(self) -> &'static str {
        match current() {
            Lang::Zh => self.zh(),
            Lang::En => self.en(),
        }
    }
}

pub fn t(m: Msg) -> &'static str {
    m.text()
}

/// Human-facing field label for a JSON key. Unknown keys fall back to the key
/// itself so new probes never render blank. JSON keys are never translated.
pub fn label(key: &str) -> &str {
    match current() {
        Lang::Zh => label_zh(key).unwrap_or(key),
        Lang::En => key,
    }
}

fn label_zh(key: &str) -> Option<&'static str> {
    Some(match key {
        // sections and machine identity
        "cpu" => "处理器",
        "memory" => "内存",
        "storage" => "存储",
        "battery" => "电池",
        "display" => "显示器",
        "audio" => "声卡",
        "network" => "网卡",
        "thermal" => "传感器",
        "vendor" => "厂商",
        "product_name" => "产品名称",
        "product_version" => "产品版本",
        "product_family" => "产品系列",
        "product_sku" => "产品 SKU",
        "board_name" => "主板型号",
        "board_version" => "主板版本",
        "bios_version" => "BIOS 版本",
        "bios_date" => "BIOS 日期",
        "ec_firmware_release" => "EC 固件版本",
        "chassis_type" => "机箱类型",
        "serial" | "serial_number" => "序列号",
        "uuid" => "UUID",
        // cpu
        "model" => "型号",
        "model_name" => "型号名称",
        "logical_cpus" => "逻辑处理器数",
        "cores" => "物理核心数",
        "siblings" => "同级线程数",
        "hyper_threading" => "超线程",
        // memory
        "total" => "总量",
        "available" => "可用",
        "used" => "已用",
        "swap_total" => "Swap 总量",
        "swap_free" => "Swap 可用",
        "speed_part_form" => "频率/型号/形态",
        // storage
        "name" => "名称",
        "size" => "容量",
        "rotational" => "机械硬盘",
        "temperature_c" => "温度 (℃)",
        // battery
        "manufacturer" => "厂商",
        "technology" => "电池类型",
        "energy_full" => "当前满容量",
        "energy_full_design" => "设计容量",
        "health_percent" => "健康度 (%)",
        "capacity_percent" => "容量 (%)",
        "status" => "状态",
        "cycle_count" => "循环次数",
        "charge_types" => "充电模式",
        "options" => "可选",
        "active" => "当前",
        "threshold" => "阈值",
        // display
        "connector" => "接口",
        "resolution" => "分辨率",
        "product_code" => "产品代码",
        "size_cm" => "尺寸 (cm)",
        "diagonal_inch" => "对角线 (英寸)",
        // thermal
        "temps" => "温度",
        "fans" => "风扇",
        "label" => "名称",
        "celsius" => "温度 (℃)",
        "rpm" => "转速 (RPM)",
        // audio / network
        "card" => "声卡",
        "id" => "ID",
        "platform_driver" => "平台驱动",
        "pci_slot" => "PCI 地址",
        "pci_id" => "PCI ID",
        "pci_driver" => "PCI 驱动",
        "class" => "类别",
        "interface" => "接口",
        "virtual" => "虚拟",
        "driver" => "驱动",
        "wireless" => "无线",
        "operstate" => "链路状态",
        "mac" => "MAC 地址",
        "mtu" => "MTU",
        "speed_mbps" => "速率 (Mbps)",
        // performance
        "platform_profile" => "平台性能档",
        "choices" => "可选档位",
        "provider" => "提供方",
        "bios_performance_enhanced" => "BIOS 性能增强",
        "power_profiles_daemon" => "PPD 服务",
        "service" => "服务",
        "state" => "状态",
        "manages_platform_profile" => "管理平台性能档",
        "note" => "说明",
        // charge / power / lid
        "ec_device" => "EC 设备",
        "usb_charging" => "USB 充电",
        "usb_charge_threshold" => "USB 充电阈值",
        "conservation_mode" => "电池养护模式",
        "camera_power" => "摄像头供电",
        "fn_lock" => "Fn 锁定",
        "fan_mode" => "风扇模式",
        "flip_to_start" => "开盖开机",
        "instant_boot" => "即时启动",
        "supported" => "是否支持",
        // io / firmware
        "io" => "IO 开关",
        "backend" => "后端",
        "io_allowlist" => "IO 白名单",
        "attributes" => "属性",
        "display_name" => "显示名称",
        "type" => "类型",
        "possible_values" => "可选值",
        "current_value" => "当前值",
        "io_switch" => "IO 开关项",
        "fwupd" => "fwupd",
        _ => return None,
    })
}

/// Localized boolean for human output (`true`/`false` vs `是`/`否`).
pub fn bool_text(b: bool) -> &'static str {
    match (current(), b) {
        (Lang::Zh, true) => "是",
        (Lang::Zh, false) => "否",
        (Lang::En, true) => "true",
        (Lang::En, false) => "false",
    }
}

/// Localized battery charge mode. Performance-profile values are intentionally
/// not translated: their Chinese names differ per model,
/// so the raw kernel value is kept.
pub fn charge_mode(v: &str) -> &str {
    match current() {
        Lang::Zh => charge_mode_zh(v).unwrap_or(v),
        Lang::En => v,
    }
}

fn charge_mode_zh(v: &str) -> Option<&'static str> {
    Some(match v {
        "Fast" => "快充",
        "Standard" => "常规",
        "Long_Life" => "养护",
        _ => return None,
    })
}

/// Fill `{name}` placeholders in a localized message.
pub fn t_args(m: Msg, args: &[(&str, &str)]) -> String {
    let mut s = t(m).to_string();
    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

#[cfg(test)]
const ALL: [Msg; 35] = [
    Msg::About,
    Msg::AfterHelp,
    Msg::HelpInfo,
    Msg::HelpConfig,
    Msg::HelpFirmware,
    Msg::HelpJson,
    Msg::HelpNoColor,
    Msg::HelpVerbose,
    Msg::HelpSysfsRoot,
    Msg::HelpLang,
    Msg::Unknown,
    Msg::Unavailable,
    Msg::NodeMissing,
    Msg::NeedRoot,
    Msg::UseRoot,
    Msg::NotNumber,
    Msg::NotBool,
    Msg::FirmwareNote,
    Msg::BiosEnhancedNote,
    Msg::PpdNote,
    Msg::PpdErr,
    Msg::NoBattery,
    Msg::ChargeThreshold,
    Msg::ChargeNote,
    Msg::UsbThreshold,
    Msg::UsbNote,
    Msg::LidFlipMissing,
    Msg::LidInstantMissing,
    Msg::LidNote,
    Msg::NoVpc,
    Msg::IoNote,
    Msg::DmiSpeed,
    Msg::ErrorPrefix,
    Msg::UnknownInfoItem,
    Msg::UnknownConfigItem,
];

/// Build the clap command with help text in the current language.
pub fn command() -> clap::Command {
    let mut cmd = Cli::command()
        .about(t(Msg::About))
        .after_help(t(Msg::AfterHelp));
    for id in [
        "info",
        "config",
        "firmware",
        "json",
        "no_color",
        "verbose",
        "sysfs_root",
        "lang",
    ] {
        let msg = match id {
            "info" => Msg::HelpInfo,
            "config" => Msg::HelpConfig,
            "firmware" => Msg::HelpFirmware,
            "json" => Msg::HelpJson,
            "no_color" => Msg::HelpNoColor,
            "verbose" => Msg::HelpVerbose,
            "sysfs_root" => Msg::HelpSysfsRoot,
            _ => Msg::HelpLang,
        };
        cmd = cmd.mut_arg(id, |a| a.help(t(msg)));
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_message_has_both_languages() {
        for m in ALL {
            assert!(!m.zh().is_empty(), "{m:?} zh empty");
            assert!(!m.en().is_empty(), "{m:?} en empty");
        }
    }

    #[test]
    fn translations_actually_differ() {
        for m in ALL {
            assert_ne!(m.zh(), m.en(), "{m:?} has identical text");
        }
    }

    #[test]
    fn locale_detection() {
        assert_eq!(lang_of_locale(Some("zh_CN.UTF-8")), Lang::Zh);
        assert_eq!(lang_of_locale(Some("zh-TW")), Lang::Zh);
        assert_eq!(lang_of_locale(Some("en_US.UTF-8")), Lang::En);
        assert_eq!(lang_of_locale(Some("C")), Lang::En);
        assert_eq!(lang_of_locale(None), Lang::En);
    }

    #[test]
    fn explicit_language_wins() {
        assert_eq!(resolve(LangArg::Zh), Lang::Zh);
        assert_eq!(resolve(LangArg::En), Lang::En);
    }

    #[test]
    fn core_keys_have_nonempty_chinese_labels() {
        for k in [
            "board_name",
            "bios_version",
            "chassis_type",
            "serial",
            "cores",
            "hyper_threading",
            "swap_total",
            "rotational",
            "temperature_c",
            "cycle_count",
            "health_percent",
            "charge_types",
            "resolution",
            "size_cm",
            "diagonal_inch",
            "celsius",
            "rpm",
            "pci_id",
            "pci_slot",
            "platform_driver",
            "operstate",
            "speed_mbps",
            "platform_profile",
            "bios_performance_enhanced",
            "usb_charging",
            "conservation_mode",
            "flip_to_start",
            "supported",
            "io_allowlist",
            "current_value",
        ] {
            let l = label_zh(k);
            assert!(l.is_some(), "{k} has no zh label");
            assert!(!l.unwrap().is_empty(), "{k} zh label is empty");
        }
    }

    #[test]
    fn unknown_key_falls_back_to_itself() {
        assert!(label_zh("no_such_key").is_none());
    }

    #[test]
    fn battery_charge_modes_are_named() {
        assert_eq!(charge_mode_zh("Fast"), Some("快充"));
        assert_eq!(charge_mode_zh("Standard"), Some("常规"));
        assert_eq!(charge_mode_zh("Long_Life"), Some("养护"));
        assert_eq!(charge_mode_zh("performance"), None);
    }
}
