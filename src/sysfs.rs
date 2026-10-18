use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

/// Read-only view over a sysfs/procfs tree. The root is configurable so the
/// same probes can run against fixtures in tests.
#[derive(Clone, Debug)]
pub struct Sysfs {
    root: PathBuf,
}

#[derive(Clone, Debug)]
pub enum Read {
    Ok(String),
    NotFound,
    Denied,
    Err(String),
}

impl Read {
    pub fn map_ok<F: FnOnce(String) -> String>(self, f: F) -> Read {
        match self {
            Read::Ok(s) => Read::Ok(f(s)),
            other => other,
        }
    }

    pub fn ok(self) -> Option<String> {
        match self {
            Read::Ok(s) => Some(s),
            _ => None,
        }
    }
}

impl Sysfs {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn path(&self, p: &str) -> PathBuf {
        let rel = p.trim_start_matches('/');
        if self.root == Path::new("/") {
            PathBuf::from(format!("/{rel}"))
        } else {
            self.root.join(rel)
        }
    }

    pub fn read(&self, p: &str) -> Read {
        match fs::read_to_string(self.path(p)) {
            Ok(s) => Read::Ok(s),
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => Read::NotFound,
                io::ErrorKind::PermissionDenied => Read::Denied,
                _ => Read::Err(e.to_string()),
            },
        }
    }

    pub fn read_trim(&self, p: &str) -> Read {
        self.read(p).map_ok(|s| s.trim().to_string())
    }

    pub fn bytes(&self, p: &str) -> Result<Vec<u8>, Read> {
        match fs::read(self.path(p)) {
            Ok(b) => Ok(b),
            Err(e) => Err(match e.kind() {
                io::ErrorKind::NotFound => Read::NotFound,
                io::ErrorKind::PermissionDenied => Read::Denied,
                _ => Read::Err(e.to_string()),
            }),
        }
    }

    pub fn exists(&self, p: &str) -> bool {
        self.path(p).exists()
    }

    /// Resolve symlinks under the sysfs root (used to find a device's PCI ancestor).
    pub fn canonical(&self, p: &str) -> Option<String> {
        fs::canonicalize(self.path(p))
            .ok()
            .map(|pb| pb.to_string_lossy().into_owned())
    }

    /// Sorted directory entries (names only). Missing/unreadable dirs are empty.
    pub fn entries(&self, p: &str) -> Vec<String> {
        let mut v: Vec<String> = match fs::read_dir(self.path(p)) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect(),
            Err(_) => Vec::new(),
        };
        v.sort();
        v
    }
}

/// A leaf value for JSON/human output. On permission errors it carries a hint.
pub fn field(r: Read) -> Value {
    match r {
        Read::Ok(s) => json!(s),
        Read::NotFound => json!({"value": null, "reason": "节点不存在"}),
        Read::Denied => {
            json!({"value": null, "reason": "需要管理员权限", "hint": "请使用 root 或 sudo 执行"})
        }
        Read::Err(e) => json!({"value": null, "reason": e}),
    }
}

pub fn field_num(r: Read) -> Value {
    match r {
        Read::Ok(s) => match s.trim().parse::<i64>() {
            Ok(n) => json!(n),
            Err(_) => json!({"value": s, "reason": "无法解析为数字"}),
        },
        other => field(other),
    }
}

pub fn field_bool(r: Read) -> Value {
    match r {
        Read::Ok(s) => match s.trim().to_ascii_lowercase().as_str() {
            "1" | "y" | "yes" | "true" | "enabled" | "enable" | "on" => json!(true),
            "0" | "n" | "no" | "false" | "disabled" | "disable" | "off" => json!(false),
            _ => json!({"value": s, "reason": "无法解析为布尔值"}),
        },
        other => field(other),
    }
}

pub fn unsupported(reason: &str) -> Value {
    json!({"value": null, "reason": reason})
}

pub fn fmt_bytes(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.2} GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.1} MiB", b / MIB)
    } else {
        format!("{bytes} B")
    }
}
