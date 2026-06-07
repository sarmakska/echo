use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::types::Turn;
use crate::{MemoryError, MemoryStore};

impl MemoryStore {
    /// Append a turn to the episode file for `day` (format "YYYY/MM/DD"),
    /// creating intermediate directories. One JSON object per line (JSONL).
    pub fn append_turn(&self, day: &str, turn: &Turn) -> Result<(), MemoryError> {
        let path = self.root().join("episodes").join(format!("{day}.jsonl"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| MemoryError::Io {
                path: parent.display().to_string(),
                source,
            })?;
        }
        let line = serde_json::to_string(turn).map_err(|e| MemoryError::Parse(e.to_string()))?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|source| MemoryError::Io {
                path: path.display().to_string(),
                source,
            })?;
        writeln!(f, "{line}").map_err(|source| MemoryError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Ok(())
    }

    /// Return the last `limit` turns recorded for `day`, in chronological order.
    /// A missing episode file yields an empty vec (not an error).
    pub fn recent_turns(&self, day: &str, limit: usize) -> Result<Vec<Turn>, MemoryError> {
        let path = self.root().join("episodes").join(format!("{day}.jsonl"));
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(MemoryError::Io {
                    path: path.display().to_string(),
                    source,
                })
            }
        };
        let mut turns = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let turn: Turn =
                serde_json::from_str(line).map_err(|e| MemoryError::Parse(e.to_string()))?;
            turns.push(turn);
        }
        if turns.len() > limit {
            turns = turns.split_off(turns.len() - limit);
        }
        Ok(turns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(tag: &str) -> MemoryStore {
        let dir = std::env::temp_dir().join(format!("echo-mem-ep-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        MemoryStore::open(&dir).unwrap()
    }

    #[test]
    fn append_then_recent_roundtrips_in_order() {
        let s = store("order");
        s.append_turn("2026/06/07", &Turn::new("user", "hi", "t1")).unwrap();
        s.append_turn("2026/06/07", &Turn::new("echo", "hello", "t2")).unwrap();
        let turns = s.recent_turns("2026/06/07", 10).unwrap();
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].text, "hi");
        assert_eq!(turns[1].text, "hello");
    }

    #[test]
    fn recent_turns_respects_limit_keeping_latest() {
        let s = store("limit");
        for i in 0..5 {
            s.append_turn("2026/06/07", &Turn::new("user", format!("m{i}"), format!("t{i}")))
                .unwrap();
        }
        let turns = s.recent_turns("2026/06/07", 2).unwrap();
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].text, "m3");
        assert_eq!(turns[1].text, "m4");
    }

    #[test]
    fn recent_turns_missing_day_is_empty() {
        let s = store("missing");
        assert!(s.recent_turns("2099/01/01", 5).unwrap().is_empty());
    }
}
