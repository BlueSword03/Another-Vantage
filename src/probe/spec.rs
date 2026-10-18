use std::collections::HashMap;

use serde_json::{json, Value};

use super::hardware;
use super::parse_charge_types;
use crate::i18n::{t, Msg};
use crate::sysfs::{field, field_num, fmt_bytes, unsupported, Read, Sysfs};

pub fn spec(sys: &Sysfs) -> Value {
    json!({
        "cpu": cpu(sys),
        "memory": memory(sys),
        "storage": storage(sys),
        "battery": battery(sys),
        "display": display(sys),
        "audio": hardware::audio(sys),
        "network": hardware::network(sys),
        "thermal": thermal(sys),
    })
}

fn cpu(sys: &Sysfs) -> Value {
    match sys.read("/proc/cpuinfo") {
        Read::Ok(text) => {
            let model = first_after(&text, "model name");
            let logical = text
                .lines()
                .filter(|l| l.trim_start().starts_with("processor"))
                .count();
            let cores = first_after(&text, "cpu cores").and_then(|s| s.parse::<u64>().ok());
            let siblings = first_after(&text, "siblings").and_then(|s| s.parse::<u64>().ok());
            let ht = match (cores, siblings) {
                (Some(c), Some(s)) => Some(s > c),
                _ => None,
            };
            json!({
                "model": model,
                "logical_cpus": logical,
                "cores": cores,
                "siblings": siblings,
                "hyper_threading": ht,
            })
        }
        other => field(other),
    }
}

fn first_after(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        if k.trim() == key {
            Some(v.trim().to_string())
        } else {
            None
        }
    })
}

fn memory(sys: &Sysfs) -> Value {
    match sys.read("/proc/meminfo") {
        Read::Ok(text) => {
            let m = parse_meminfo(&text);
            let kib = |k: &str| m.get(k).copied().unwrap_or(0) * 1024;
            let total = kib("MemTotal");
            let avail = m
                .get("MemAvailable")
                .map(|_| kib("MemAvailable"))
                .unwrap_or_else(|| kib("MemFree"));
            let swap_total = kib("SwapTotal");
            let swap_free = kib("SwapFree");
            json!({
                "total": fmt_bytes(total),
                "available": fmt_bytes(avail),
                "used": fmt_bytes(total.saturating_sub(avail)),
                "swap_total": fmt_bytes(swap_total),
                "swap_free": fmt_bytes(swap_free),
                "speed_part_form": unsupported(t(Msg::DmiSpeed)),
            })
        }
        other => field(other),
    }
}

fn parse_meminfo(text: &str) -> HashMap<String, u64> {
    let mut m = HashMap::new();
    for l in text.lines() {
        if let Some((k, v)) = l.split_once(':') {
            let num = v
                .split_whitespace()
                .next()
                .and_then(|n| n.parse::<u64>().ok());
            if let Some(n) = num {
                m.insert(k.trim().to_string(), n);
            }
        }
    }
    m
}

fn storage(sys: &Sysfs) -> Value {
    let mut out = Vec::new();
    for name in sys.entries("/sys/block") {
        if name.starts_with("loop")
            || name.starts_with("ram")
            || name.starts_with("zram")
            || name.starts_with("dm-")
        {
            continue;
        }
        let base = format!("/sys/block/{name}");
        let sectors = sys
            .read_trim(&format!("{base}/size"))
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        let model = sys
            .read_trim(&format!("{base}/device/model"))
            .ok()
            .map(|s| s.trim().to_string());
        let vendor = sys
            .read_trim(&format!("{base}/device/vendor"))
            .ok()
            .map(|s| s.trim().to_string());
        let rotational = sys.read_trim(&format!("{base}/queue/rotational")).ok();
        let temp = disk_temp(sys, &base);
        out.push(json!({
            "name": name,
            "vendor": vendor,
            "model": model,
            "size": sectors.map(|s| fmt_bytes(s * 512)),
            "rotational": rotational.map(|r| r == "1"),
            "temperature_c": temp,
        }));
    }
    json!(out)
}

fn disk_temp(sys: &Sysfs, base: &str) -> Option<f64> {
    let read = |p: String| sys.read_trim(&p).ok().and_then(|s| s.parse::<f64>().ok());
    // NVMe exposes hwmonN directly under the controller device.
    for entry in sys.entries(&format!("{base}/device")) {
        if entry.starts_with("hwmon") {
            if let Some(v) = read(format!("{base}/device/{entry}/temp1_input")) {
                return Some((v / 1000.0 * 10.0).round() / 10.0);
            }
        }
    }
    // SATA exposes device/hwmon/hwmonN.
    for h in sys.entries(&format!("{base}/device/hwmon")) {
        if let Some(v) = read(format!("{base}/device/hwmon/{h}/temp1_input")) {
            return Some((v / 1000.0 * 10.0).round() / 10.0);
        }
    }
    None
}

fn battery(sys: &Sysfs) -> Value {
    let mut out = Vec::new();
    for name in sys.entries("/sys/class/power_supply") {
        if !name.starts_with("BAT") {
            continue;
        }
        let b = format!("/sys/class/power_supply/{name}");
        let energy_full = sys
            .read_trim(&format!("{b}/energy_full"))
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        let design = sys
            .read_trim(&format!("{b}/energy_full_design"))
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        let health = match (energy_full, design) {
            (Some(f), Some(d)) if d > 0 => Some((f as f64 / d as f64 * 1000.0).round() / 10.0),
            _ => None,
        };
        let (options, active) = match sys.read_trim(&format!("{b}/charge_types")) {
            Read::Ok(s) => parse_charge_types(Some(s.as_str())),
            _ => (Vec::new(), None),
        };
        out.push(json!({
            "name": name,
            "manufacturer": field(sys.read_trim(&format!("{b}/manufacturer"))),
            "model_name": field(sys.read_trim(&format!("{b}/model_name"))),
            "technology": field(sys.read_trim(&format!("{b}/technology"))),
            "serial_number": field(sys.read_trim(&format!("{b}/serial_number"))),
            "energy_full": energy_full.map(|v| format!("{:.2} Wh", v as f64 / 1e6)),
            "energy_full_design": design.map(|v| format!("{:.2} Wh", v as f64 / 1e6)),
            "health_percent": health,
            "capacity_percent": field_num(sys.read_trim(&format!("{b}/capacity"))),
            "status": field(sys.read_trim(&format!("{b}/status"))),
            "cycle_count": field_num(sys.read_trim(&format!("{b}/cycle_count"))),
            "charge_types": {"options": options, "active": active},
        }));
    }
    json!(out)
}

fn display(sys: &Sysfs) -> Value {
    let mut out = Vec::new();
    for name in sys.entries("/sys/class/drm") {
        if !name.starts_with("card") || !name.contains('-') {
            continue;
        }
        let base = format!("/sys/class/drm/{name}");
        let status = sys.read_trim(&format!("{base}/status")).ok();
        if status.as_deref() != Some("connected") {
            continue;
        }
        let resolution = sys
            .read_trim(&format!("{base}/modes"))
            .ok()
            .and_then(|m| m.lines().next().map(|s| s.to_string()));
        let parsed = sys
            .bytes(&format!("{base}/edid"))
            .ok()
            .as_deref()
            .and_then(parse_edid);
        out.push(json!({
            "connector": name,
            "status": status,
            "resolution": resolution,
            "manufacturer": parsed.as_ref().and_then(|p| p.manufacturer.clone()),
            "product_code": parsed.as_ref().and_then(|p| p.product_code.clone()),
            "size_cm": parsed.as_ref().and_then(|p| p.size_cm.clone()),
            "diagonal_inch": parsed.as_ref().and_then(|p| p.diagonal_inch),
        }));
    }
    json!(out)
}

struct Edid {
    manufacturer: Option<String>,
    product_code: Option<String>,
    size_cm: Option<String>,
    diagonal_inch: Option<f64>,
}

fn parse_edid(b: &[u8]) -> Option<Edid> {
    if b.len() < 128 {
        return None;
    }
    let m = ((b[8] as u16) << 8) | b[9] as u16;
    let letters = [
        ((m >> 10) & 0x1f) as u8,
        ((m >> 5) & 0x1f) as u8,
        (m & 0x1f) as u8,
    ];
    let manufacturer = if letters.iter().all(|&c| (1..=26).contains(&c)) {
        Some(letters.iter().map(|&c| (b'A' + c - 1) as char).collect())
    } else {
        None
    };
    let product = ((b[11] as u16) << 8) | b[10] as u16;
    let (w, h) = (b[21] as u32, b[22] as u32);
    let (size_cm, diagonal_inch) = if w > 0 && h > 0 {
        let d = ((w * w + h * h) as f64).sqrt() / 2.54;
        (
            Some(format!("{w} x {h} cm")),
            Some((d * 10.0).round() / 10.0),
        )
    } else {
        (None, None)
    };
    Some(Edid {
        manufacturer,
        product_code: Some(format!("{product} (0x{product:04X})")),
        size_cm,
        diagonal_inch,
    })
}

fn thermal(sys: &Sysfs) -> Value {
    let mut out = Vec::new();
    for h in sys.entries("/sys/class/hwmon") {
        if !h.starts_with("hwmon") {
            continue;
        }
        let base = format!("/sys/class/hwmon/{h}");
        let name = sys
            .read_trim(&format!("{base}/name"))
            .ok()
            .unwrap_or_else(|| h.clone());

        let mut temps = Vec::new();
        let mut fans = Vec::new();
        for f in sys.entries(&base) {
            if let Some(idx) = f
                .strip_prefix("temp")
                .and_then(|s| s.strip_suffix("_input"))
            {
                if let Some(v) = sys
                    .read_trim(&format!("{base}/{f}"))
                    .ok()
                    .and_then(|s| s.parse::<f64>().ok())
                {
                    let label = sys.read_trim(&format!("{base}/temp{idx}_label")).ok();
                    temps.push(json!({
                        "label": label,
                        "celsius": (v / 1000.0 * 10.0).round() / 10.0,
                    }));
                }
            }
            if let Some(idx) = f.strip_prefix("fan").and_then(|s| s.strip_suffix("_input")) {
                if let Some(v) = sys
                    .read_trim(&format!("{base}/{f}"))
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                {
                    let label = sys.read_trim(&format!("{base}/fan{idx}_label")).ok();
                    fans.push(json!({"label": label, "rpm": v}));
                }
            }
        }
        out.push(json!({"name": name, "temps": temps, "fans": fans}));
    }
    json!(out)
}

#[cfg(test)]
mod tests {
    use super::parse_edid;

    fn synthetic_edid() -> Vec<u8> {
        let mut b = vec![0u8; 128];
        // Manufacturer "CSO": (3 << 10) | (19 << 5) | 15 = 0x0E6F
        b[8] = 0x0E;
        b[9] = 0x6F;
        // Product code 0x160C (= 5644, matching this machine), little-endian.
        b[10] = 0x0C;
        b[11] = 0x16;
        // 34 x 22 cm.
        b[21] = 34;
        b[22] = 22;
        b
    }

    #[test]
    fn parses_edid_identity_and_size() {
        let edid = parse_edid(&synthetic_edid()).expect("edid");
        assert_eq!(edid.manufacturer.as_deref(), Some("CSO"));
        assert_eq!(edid.product_code.as_deref(), Some("5644 (0x160C)"));
        assert_eq!(edid.size_cm.as_deref(), Some("34 x 22 cm"));
        let d = edid.diagonal_inch.expect("diagonal");
        assert!((15.8..16.0).contains(&d), "diagonal was {d}");
    }

    #[test]
    fn rejects_short_edid() {
        assert!(parse_edid(&[0u8; 16]).is_none());
    }
}
