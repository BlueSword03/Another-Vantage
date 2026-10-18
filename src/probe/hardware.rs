use serde_json::{json, Value};

use crate::sysfs::{field, field_num, Read, Sysfs};

/// Audio and network controllers. Model names come from `lspci -mm -nn`
/// (the PCI ID database); per-device state comes from sysfs.
pub fn audio(sys: &Sysfs) -> Value {
    let lspci = Lspci::load();
    let mut out = Vec::new();
    for card in sys.entries("/sys/class/sound") {
        if !card.starts_with("card") {
            continue;
        }
        let base = format!("/sys/class/sound/{card}");
        let id = sys.read_trim(&format!("{base}/id")).ok();
        // The sound card sits on a platform device; its PCI ancestor is the
        // HDA controller users actually care about.
        let (platform_driver, _, _) = device_uevent(sys, &format!("{base}/device/uevent"));
        let slot = sys
            .canonical(&format!("{base}/device"))
            .as_deref()
            .and_then(find_pci_slot);
        let entry = slot.as_deref().and_then(|s| lspci.find(s));
        let lspci_id = entry.and_then(|e| e.pci_id.clone());
        let (pci_id, pci_driver) = match slot.as_deref() {
            Some(s) => pci_uevent(sys, s),
            None => (None, None),
        };
        out.push(json!({
            "card": card,
            "id": id,
            "platform_driver": platform_driver,
            "pci_slot": slot,
            "pci_id": pci_id.or(lspci_id),
            "pci_driver": pci_driver,
            "class": entry.map(|e| e.class.clone()),
            "model": entry.map(|e| format!("{} {}", e.vendor, e.device)),
        }));
    }
    json!(out)
}

pub fn network(sys: &Sysfs) -> Value {
    let lspci = Lspci::load();
    let mut out = Vec::new();
    for iface in sys.entries("/sys/class/net") {
        let base = format!("/sys/class/net/{iface}");
        let (driver, pci_id, slot) = device_uevent(sys, &format!("{base}/device/uevent"));
        let entry = slot.as_deref().and_then(|s| lspci.find(s));
        let lspci_id = entry.and_then(|e| e.pci_id.clone());
        let speed = sys
            .read_trim(&format!("{base}/speed"))
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .filter(|v| *v > 0);
        out.push(json!({
            "interface": iface,
            "virtual": slot.is_none(),
            "driver": driver,
            "pci_id": pci_id.or(lspci_id),
            "pci_slot": slot,
            "class": entry.map(|e| e.class.clone()),
            "model": entry.map(|e| format!("{} {}", e.vendor, e.device)),
            "wireless": sys.exists(&format!("{base}/wireless")),
            "operstate": field(sys.read_trim(&format!("{base}/operstate"))),
            "mac": field(sys.read_trim(&format!("{base}/address"))),
            "mtu": field_num(sys.read_trim(&format!("{base}/mtu"))),
            "speed_mbps": speed,
        }));
    }
    json!(out)
}

fn device_uevent(sys: &Sysfs, path: &str) -> (Option<String>, Option<String>, Option<String>) {
    match sys.read(path) {
        Read::Ok(text) => {
            let get = |key: &str| {
                text.lines()
                    .find_map(|l| l.strip_prefix(key))
                    .map(|s| s.trim().to_string())
            };
            (get("DRIVER="), get("PCI_ID="), get("PCI_SLOT_NAME="))
        }
        _ => (None, None, None),
    }
}

fn pci_uevent(sys: &Sysfs, slot: &str) -> (Option<String>, Option<String>) {
    match sys.read(&format!("/sys/bus/pci/devices/{slot}/uevent")) {
        Read::Ok(text) => {
            let get = |key: &str| {
                text.lines()
                    .find_map(|l| l.strip_prefix(key))
                    .map(|s| s.trim().to_string())
            };
            (get("PCI_ID="), get("DRIVER="))
        }
        _ => (None, None),
    }
}

/// Find a `dddd:dd:dd.d` component in a resolved device path.
fn find_pci_slot(path: &str) -> Option<String> {
    path.split('/')
        .find(|c| is_pci_slot(c))
        .map(|s| s.to_string())
}

fn is_pci_slot(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 12
        && b[4] == b':'
        && b[7] == b':'
        && b[10] == b'.'
        && [0, 1, 2, 3, 5, 6, 8, 9, 11]
            .iter()
            .all(|&i| (b[i] as char).is_ascii_hexdigit())
}

#[derive(Debug, Clone)]
struct LspciEntry {
    slot: String,
    class: String,
    vendor: String,
    device: String,
    pci_id: Option<String>,
}

struct Lspci {
    entries: Vec<LspciEntry>,
}

impl Lspci {
    fn load() -> Self {
        let entries = std::process::Command::new("lspci")
            .args(["-mm", "-nn"])
            .output()
            .ok()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter_map(parse_lspci_mm)
                    .collect()
            })
            .unwrap_or_default();
        Self { entries }
    }

    fn find(&self, slot: &str) -> Option<&LspciEntry> {
        self.entries.iter().find(|e| same_slot(&e.slot, slot))
    }
}

fn same_slot(a: &str, b: &str) -> bool {
    let norm = |s: &str| s.trim_start_matches("0000:").to_lowercase();
    norm(a) == norm(b)
}

/// Parse one `lspci -mm -nn` line, e.g.
/// `00:1f.3 "Multimedia audio controller [0401]" "Intel Corporation [8086]" "Alder Lake PCH-P High Definition Audio Controller [8086:51c8]" ...`
fn parse_lspci_mm(line: &str) -> Option<LspciEntry> {
    let (slot, mut rest) = line.trim().split_once(' ')?;
    let mut fields = Vec::new();
    while let Some(stripped) = rest.trim_start().strip_prefix('"') {
        let (value, after) = stripped.split_once('"')?;
        fields.push(value.to_string());
        rest = after;
    }
    if fields.len() < 3 {
        return None;
    }
    Some(LspciEntry {
        slot: slot.to_string(),
        class: strip_pci_id(&fields[0]),
        vendor: strip_pci_id(&fields[1]),
        device: strip_pci_id(&fields[2]),
        pci_id: extract_pci_id(&fields[2]),
    })
}

/// Remove a trailing `[xxxx]` / `[xxxx:yyyy]` suffix that `-nn` appends.
fn strip_pci_id(s: &str) -> String {
    match s.rfind(" [") {
        Some(i) if s.ends_with(']') => s[..i].to_string(),
        _ => s.to_string(),
    }
}

/// Extract `xxxx:yyyy` from a `-nn` device field.
fn extract_pci_id(s: &str) -> Option<String> {
    let inner = s.rfind('[').and_then(|i| s[i + 1..].strip_suffix(']'))?;
    let b = inner.as_bytes();
    if b.len() == 9
        && b[4] == b':'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || (*c as char).is_ascii_hexdigit())
    {
        Some(inner.to_ascii_lowercase())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_pci_id, find_pci_slot, is_pci_slot, parse_lspci_mm, same_slot};

    #[test]
    fn parses_lspci_mm_line() {
        let line = r#"00:1f.3 "Multimedia audio controller [0401]" "Intel Corporation [8086]" "Alder Lake PCH-P High Definition Audio Controller [8086:51c8]" -r01 -p00 "Lenovo [17aa]" "Device [3802]""#;
        let e = parse_lspci_mm(line).expect("entry");
        assert_eq!(e.slot, "00:1f.3");
        assert_eq!(e.class, "Multimedia audio controller");
        assert_eq!(e.vendor, "Intel Corporation");
        assert_eq!(
            e.device,
            "Alder Lake PCH-P High Definition Audio Controller"
        );
        assert_eq!(e.pci_id.as_deref(), Some("8086:51c8"));
    }

    #[test]
    fn normalizes_pci_domain() {
        assert!(same_slot("00:1f.3", "0000:00:1f.3"));
        assert!(!same_slot("00:1f.3", "0000:2c:00.0"));
    }

    #[test]
    fn finds_pci_slot_in_device_path() {
        assert_eq!(
            find_pci_slot("/sys/devices/pci0000:00/0000:00:1f.3/skl_hda_dsp_generic").as_deref(),
            Some("0000:00:1f.3")
        );
        assert_eq!(find_pci_slot("/sys/devices/virtual/sound/card0"), None);
        assert!(is_pci_slot("0000:2c:00.0"));
        assert!(!is_pci_slot("pci0000:00"));
    }

    #[test]
    fn extracts_pci_id_or_none() {
        assert_eq!(
            extract_pci_id("Alder Lake PCH-P High Definition Audio Controller [8086:51c8]")
                .as_deref(),
            Some("8086:51c8")
        );
        assert_eq!(extract_pci_id("Intel Corporation [8086]"), None);
    }

    #[test]
    fn rejects_short_lines() {
        assert!(parse_lspci_mm("00:1f.3 \"only class\"").is_none());
    }
}
