use std::fs;

use crate::{MemoryError, MemoryStore};

impl MemoryStore {
    fn digests_dir(&self) -> std::path::PathBuf {
        self.root().join("digests")
    }

    /// Save a PreSession digest as `digests/<name>.md`. `name` should sort
    /// chronologically (e.g. "session_2026-06-07-1432").
    pub fn save_digest(&self, name: &str, content: &str) -> Result<(), MemoryError> {
        let dir = self.digests_dir();
        fs::create_dir_all(&dir).map_err(|source| MemoryError::Io {
            path: dir.display().to_string(),
            source,
        })?;
        let path = dir.join(format!("{name}.md"));
        fs::write(&path, content).map_err(|source| MemoryError::Io {
            path: path.display().to_string(),
            source,
        })
    }

    /// Return the content of the lexicographically-last digest (the most recent
    /// session, given chronological names). No digests → Ok(None).
    pub fn latest_digest(&self) -> Result<Option<String>, MemoryError> {
        let dir = self.digests_dir();
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(MemoryError::Io {
                    path: dir.display().to_string(),
                    source,
                })
            }
        };
        let mut names: Vec<String> = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| MemoryError::Io {
                path: dir.display().to_string(),
                source,
            })?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                names.push(name);
            }
        }
        names.sort();
        match names.last() {
            None => Ok(None),
            Some(latest) => {
                let path = dir.join(latest);
                let content = fs::read_to_string(&path).map_err(|source| MemoryError::Io {
                    path: path.display().to_string(),
                    source,
                })?;
                Ok(Some(content))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(tag: &str) -> MemoryStore {
        let dir = std::env::temp_dir().join(format!("echo-mem-dig-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        MemoryStore::open(&dir).unwrap()
    }

    #[test]
    fn latest_returns_most_recent_by_name() {
        let s = store("latest");
        s.save_digest("session_2026-06-06-0900", "older").unwrap();
        s.save_digest("session_2026-06-07-1432", "newer").unwrap();
        assert_eq!(s.latest_digest().unwrap().unwrap(), "newer");
    }

    #[test]
    fn latest_with_no_digests_is_none() {
        let s = store("empty");
        assert!(s.latest_digest().unwrap().is_none());
    }
}
