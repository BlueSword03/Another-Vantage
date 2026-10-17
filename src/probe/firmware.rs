use serde_json::{json, Value};

use crate::sysfs::{field, Sysfs};

const DMI: &str = "/sys/devices/virtual/dmi/id";

pub fn firmware(sys: &Sysfs) -> Value {
    json!({
        "bios_version": field(sys.read_trim(&format!("{DMI}/bios_version"))),
        "bios_date": field(sys.read_trim(&format!("{DMI}/bios_date"))),
        "ec_firmware_release": field(sys.read_trim(&format!("{DMI}/ec_firmware_release"))),
        "fwupd": {
            "available": which("fwupdmgr").is_some(),
            "note": "运行 `fwupdmgr get-updates` 查看可用更新；21CY 的 BIOS1/System Firmware 标记为“固件更新不可用”，无 LVFS 刷写路径",
        },
    })
}

fn which(cmd: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|p| p.join(cmd))
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
}
