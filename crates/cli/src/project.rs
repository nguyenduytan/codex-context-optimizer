use crate::storage;
use anyhow::Result;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Component, Path},
};

const MAX_ENTRIES: usize = 20000;
const EXCLUDED: &[&str] = &[
    ".git",
    ".ctxc",
    ".tools",
    "node_modules",
    "target",
    "dist",
    "build",
    "coverage",
    "vendor",
    ".cache",
    ".next",
    "generated",
    "__pycache__",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMap {
    pub version: u32,
    pub fingerprint: String,
    pub content: String,
    pub truncated: bool,
    pub cached: bool,
    pub limit: usize,
    pub entries_examined: usize,
    pub directory_stamps: BTreeMap<String, u128>,
}

fn safe_relative(path: &Path) -> bool {
    for part in path.components() {
        let Component::Normal(name) = part else {
            return false;
        };
        let name = name.to_string_lossy().to_lowercase();
        if EXCLUDED.contains(&name.as_str())
            || name.starts_with('.')
            || name.contains("credential")
            || name.contains("secret")
            || name.starts_with("id_rsa")
            || name.starts_with("id_ed25519")
            || name.ends_with(".pem")
            || name.ends_with(".key")
            || name.ends_with(".p12")
        {
            return false;
        }
    }
    true
}
fn stamp(path: &Path) -> u128 {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|t| t.as_nanos())
        .unwrap_or(0)
}

pub fn build(root: &Path, limit: usize, cache: bool) -> Result<ProjectMap> {
    let cache_path = root.join(".ctxc/project-map.json");
    storage::ensure_safe(root, &cache_path)?;
    if cache && cache_path.is_file() && fs::metadata(&cache_path)?.len() <= 4 * 1024 * 1024 {
        if let Ok(mut map) = serde_json::from_slice::<ProjectMap>(&fs::read(&cache_path)?) {
            let valid = map.version == 1
                && !map.truncated
                && map.limit == limit
                && map.content.len() <= limit
                && !map.directory_stamps.is_empty()
                && map.directory_stamps.len() <= MAX_ENTRIES
                && map.directory_stamps.iter().all(|(p, t)| {
                    // Cache content is untrusted. Only stat contained, non-symlink paths.
                    let rel = Path::new(p);
                    (p.is_empty() || rel.components().all(|c| matches!(c, Component::Normal(_))))
                        && storage::ensure_safe(root, &root.join(rel)).is_ok()
                        && stamp(&root.join(rel)) == *t
                });
            if valid {
                map.cached = true;
                return Ok(map);
            }
        }
    }
    let mut paths = vec![];
    let mut stamps = BTreeMap::new();
    stamps.insert(String::new(), stamp(root));
    // Ignore changes can alter visibility without changing directory mtimes.
    for p in [".gitignore", ".ignore", ".git/info/exclude"] {
        stamps.insert(p.into(), stamp(&root.join(p)));
    }
    let root_owned = root.to_path_buf();
    let walker = WalkBuilder::new(root)
        .hidden(true)
        .follow_links(false)
        .parents(false)
        .git_global(false)
        .require_git(false)
        .sort_by_file_path(|a, b| a.cmp(b))
        .filter_entry(move |e| {
            e.path() == root_owned
                || (e.path().strip_prefix(&root_owned).is_ok_and(safe_relative)
                    && storage::ensure_safe(&root_owned, e.path()).is_ok())
        })
        .build();
    let mut examined = 0;
    let mut truncated = false;
    for entry in walker {
        if examined >= MAX_ENTRIES {
            truncated = true;
            break;
        }
        examined += 1;
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                truncated = true;
                continue;
            }
        };
        let path = entry.path();
        if path == root {
            continue;
        }
        let rel = path.strip_prefix(root)?;
        if !safe_relative(rel) {
            continue;
        }
        let Some(kind) = entry.file_type() else {
            continue;
        };
        let display = rel.to_string_lossy().replace('\\', "/");
        if kind.is_dir() {
            stamps.insert(display.clone(), stamp(path));
            for name in [".gitignore", ".ignore"] {
                let key = format!("{display}/{name}");
                stamps.insert(key, stamp(&path.join(name)));
            }
        } else if kind.is_file() {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if fs::symlink_metadata(path)?.file_attributes() & 0x400 != 0 {
                    continue;
                }
            }
            paths.push(display);
        }
    }
    paths.sort_by_key(|p| {
        (
            !is_manifest(p),
            !p.starts_with("src/") && !p.starts_with("tests/"),
            p.clone(),
        )
    });
    let mut fingerprint = Sha256::new();
    for p in &paths {
        fingerprint.update(p.as_bytes());
        fingerprint.update([0]);
    }
    let mut content = String::new();
    let mut add = |line: &str| {
        if content.len() + line.len() <= limit {
            content.push_str(line);
            true
        } else {
            false
        }
    };
    add("Project map (paths only; inspect relevant files on demand):\n");
    for p in &paths {
        if !add(&format!("- {p}\n")) {
            truncated = true;
            break;
        }
    }
    // Read no source bodies. Only known non-secret top-level manifest names give
    // conservative command suggestions; never execute a project-defined command.
    let hints = if paths.iter().any(|p| p == "Cargo.toml") {
        "Suggested validation: cargo test (narrow by package/test).\n"
    } else if paths.iter().any(|p| p == "package.json") {
        "Inspect package.json scripts before choosing a targeted test command.\n"
    } else {
        ""
    };
    if !add(hints) {
        truncated = true;
    }
    let map = ProjectMap {
        version: 1,
        fingerprint: format!("{:x}", fingerprint.finalize()),
        content,
        truncated,
        cached: false,
        limit,
        entries_examined: examined,
        directory_stamps: stamps,
    };
    if cache && root.join(".ctxc/config.toml").is_file() {
        // A truncated discovery is regenerated: unseen structure cannot be safely cached.
        if !map.truncated {
            storage::atomic_write(root, &cache_path, &serde_json::to_vec(&map)?)?;
        }
    }
    Ok(map)
}

fn is_manifest(p: &str) -> bool {
    [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "README.md",
    ]
    .contains(&p)
}

pub fn read_context_file(path: &Path, max_bytes: u64) -> Result<String> {
    let mut text = String::new();
    fs::File::open(path)?
        .take(max_bytes + 1)
        .read_to_string(&mut text)?;
    anyhow::ensure!(
        text.len() as u64 <= max_bytes,
        "input exceeds {max_bytes} bytes"
    );
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn excludes_and_bounds() {
        let d = tempfile::tempdir().unwrap();
        for p in ["src", "node_modules", "target"] {
            fs::create_dir(d.path().join(p)).unwrap();
        }
        for p in [
            "src/main.rs",
            ".env",
            "credentials.json",
            "node_modules/no.js",
            "target/no",
            "README.md",
            "ignored.txt",
        ] {
            fs::write(d.path().join(p), "secret body not read").unwrap();
        }
        fs::write(d.path().join(".gitignore"), "ignored.txt\n").unwrap();
        let m = build(d.path(), 8192, false).unwrap();
        assert!(m.content.contains("src/main.rs"));
        for s in [
            ".env",
            "credentials",
            "node_modules",
            "secret body",
            "ignored.txt",
        ] {
            assert!(!m.content.contains(s));
        }
        assert!(build(d.path(), 16, false).unwrap().content.len() <= 16);
    }
    #[test]
    fn unicode_limit() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("xin-chào.rs"), "").unwrap();
        assert!(build(d.path(), 64, false).unwrap().content.len() <= 64);
    }
}
