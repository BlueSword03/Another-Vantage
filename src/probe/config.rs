use serde_json::{json, Value};

use super::parse_charge_types;
use crate::i18n::{t, t_args, Msg};
use crate::sysfs::{field, field_bool, field_num, unsupported, Read, Sysfs};

const IO_ALLOWLIST: [&str; 10] = [
    "USBPort",
    "MemoryCardSlot",
    "IntegratedCamera",
    "Bluetooth",
    "FingerprintReader",
    "Microphone",
    "EthernetLAN",
    "Thunderbolt(TM)",
    "WirelessLAN",
    "AlwaysOnUSB",
];

pub fn performance(sys: &Sysfs) -> Value {
    let choices = match sys.read_trim("/sys/firmware/acpi/platform_profile_choices") {
        Read::Ok(s) => json!(s
            .split_whitespace()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()),
        other => field(other),
    };
    json!({
        "platform_profile": field(sys.read_trim("/sys/firmware/acpi/platform_profile")),
        "choices": choices,
        "provider": field(sys.read_trim("/sys/class/platform-profile/platform-profile-0/name")),
        "bios_performance_enhanced": unsupported(t(Msg::BiosEnhancedNote)),
        "power_profiles_daemon": ppd_status(),
        "note": t(Msg::PpdNote),
    })
}

fn ppd_status() -> Value {
    match std::process::Command::new("systemctl")
        .args(["is-active", "power-profiles-daemon"])
        .output()
    {
        Ok(o) => {
            let state = String::from_utf8_lossy(&o.stdout).trim().to_string();
            json!({
                "service": "power-profiles-daemon",
                "state": state,
                "manages_platform_profile": state == "active",
            })
        }
        Err(e) => unsupported(&t_args(Msg::PpdErr, &[("e", &e.to_string())])),
    }
}

pub fn charge(sys: &Sysfs) -> Value {
    let bats: Vec<String> = sys
        .entries("/sys/class/power_supply")
        .into_iter()
        .filter(|n| n.starts_with("BAT"))
        .collect();
    if bats.is_empty() {
        return json!({"battery": unsupported(t(Msg::NoBattery))});
    }
    let name = &bats[0];
    let b = format!("/sys/class/power_supply/{name}");
    let (options, active) = match sys.read_trim(&format!("{b}/charge_types")) {
        Read::Ok(s) => parse_charge_types(Some(s.as_str())),
        _ => (Vec::new(), None),
    };
    json!({
        "battery": name,
        "charge_types": {"options": options, "active": active},
        "status": field(sys.read_trim(&format!("{b}/status"))),
        "capacity_percent": field_num(sys.read_trim(&format!("{b}/capacity"))),
        "threshold": unsupported(t(Msg::ChargeThreshold)),
        "note": t(Msg::ChargeNote),
    })
}

pub fn power(sys: &Sysfs) -> Value {
    let ec = ec_device(sys);
    let base = ec
        .as_ref()
        .map(|d| format!("/sys/bus/platform/devices/{d}"))
        .unwrap_or_else(|| "/sys/bus/platform/devices/VPC2004:00".to_string());
    json!({
        "ec_device": ec,
        "usb_charging": field_bool(sys.read_trim(&format!("{base}/usb_charging"))),
        "usb_charge_threshold": unsupported(t(Msg::UsbThreshold)),
        "conservation_mode": field(sys.read_trim(&format!("{base}/conservation_mode"))),
        "camera_power": field_bool(sys.read_trim(&format!("{base}/camera_power"))),
        "fn_lock": field_bool(sys.read_trim(&format!("{base}/fn_lock"))),
        "fan_mode": field_num(sys.read_trim(&format!("{base}/fan_mode"))),
        "note": t(Msg::UsbNote),
    })
}

pub fn lid(sys: &Sysfs) -> Value {
    match ec_device(sys) {
        Some(dev) => {
            let base = format!("/sys/bus/platform/devices/{dev}");
            let flip = sys.exists(&format!("{base}/flip_to_start"));
            let instant = sys.exists(&format!("{base}/instant_boot"));
            json!({
                "ec_device": dev,
                "flip_to_start": if flip {
                    field_bool(sys.read_trim(&format!("{base}/flip_to_start")))
                } else {
                    unsupported(t(Msg::LidFlipMissing))
                },
                "instant_boot": if instant {
                    field_bool(sys.read_trim(&format!("{base}/instant_boot")))
                } else {
                    unsupported(t(Msg::LidInstantMissing))
                },
                "supported": flip || instant,
                "note": t(Msg::LidNote),
            })
        }
        None => json!({"supported": false, "reason": t(Msg::NoVpc)}),
    }
}

pub fn io(sys: &Sysfs) -> Value {
    let base = "/sys/class/firmware-attributes/thinklmi/attributes";
    let mut attrs: Vec<(bool, String, Value)> = Vec::new();
    for name in sys.entries(base) {
        let d = format!("{base}/{name}");
        let is_io = IO_ALLOWLIST.contains(&name.as_str());
        let entry = json!({
            "name": name,
            "display_name": field(sys.read_trim(&format!("{d}/display_name"))),
            "type": field(sys.read_trim(&format!("{d}/type"))),
            "possible_values": field(sys.read_trim(&format!("{d}/possible_values"))),
            "current_value": field(sys.read_trim(&format!("{d}/current_value"))),
        });
        attrs.push((is_io, name, entry));
    }
    // IO switches first, then alphabetical.
    attrs.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let list: Vec<Value> = attrs
        .into_iter()
        .map(|(is_io, _, mut e)| {
            if let Some(o) = e.as_object_mut() {
                o.insert("io_switch".into(), json!(is_io));
            }
            e
        })
        .collect();
    json!({
        "backend": base,
        "io_allowlist": IO_ALLOWLIST,
        "attributes": list,
        "note": t(Msg::IoNote),
    })
}

fn ec_device(sys: &Sysfs) -> Option<String> {
    sys.entries("/sys/bus/platform/devices")
        .into_iter()
        .find(|d| d.starts_with("VPC2004"))
}
