//! Echo file-based memory store. Phase 1 subset of PLAN.md §7:
//! Markdown facts, daily JSONL episodes, recency recall, PreSession digests.

mod digests;
mod episodes;
mod facts;
mod types;

use std::path::{Path, PathBuf};

pub use types::{Fact, Turn};

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("memory io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse episode line: {0}")]
    Parse(String),
}

/// Root-scoped handle to the on-disk store. Layout lives under `root`.
pub struct MemoryStore {
    root: PathBuf,
}

impl MemoryStore {
    /// Open (creating the root directory if needed) a store at `root`.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, MemoryError> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(&root).map_err(|source| MemoryError::Io {
            path: root.display().to_string(),
            source,
        })?;
        Ok(Self { root })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
}
