use serde_json::{json, Value};

use crate::sysfs::{field, Read, Sysfs};

const DMI: &str = "/sys/devices/virtual/dmi/id";

pub fn model(sys: &Sysfs) -> Value {
    let g = |n: &str| field(sys.read_trim(&format!("{DMI}/{n}")));

    let chassis = match sys.read_trim(&format!("{DMI}/chassis_type")) {
        Read::Ok(s) => {
            let name = s
                .parse::<u32>()
                .ok()
                .and_then(chassis_name)
                .unwrap_or("未知");
            json!(format!("{s} ({name})"))
        }
        other => field(other),
    };

    json!({
        "vendor": g("sys_vendor"),
        "product_name": g("product_name"),
        "product_version": g("product_version"),
        "product_family": g("product_family"),
        "product_sku": g("product_sku"),
        "board_name": g("board_name"),
        "board_version": g("board_version"),
        "bios_version": g("bios_version"),
        "bios_date": g("bios_date"),
        "ec_firmware_release": g("ec_firmware_release"),
        "chassis_type": chassis,
        "serial": g("product_serial"),
        "uuid": g("product_uuid"),
    })
}

fn chassis_name(code: u32) -> Option<&'static str> {
    Some(match code {
        1 => "Other",
        2 => "Unknown",
        3 => "Desktop",
        4 => "Low Profile Desktop",
        5 => "Pizza Box",
        6 => "Mini Tower",
        7 => "Tower",
        8 => "Portable",
        9 => "Laptop",
        10 => "Notebook",
        11 => "Hand Held",
        12 => "Docking Station",
        13 => "All In One",
        14 => "Sub Notebook",
        15 => "Space-saving",
        16 => "Lunch Box",
        17 => "Main Server Chassis",
        30 => "Tablet",
        31 => "Convertible",
        32 => "Detachable",
        _ => return None,
    })
}
