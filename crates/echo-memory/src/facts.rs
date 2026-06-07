use std::fs;

use crate::types::Fact;
use crate::{MemoryError, MemoryStore};

impl MemoryStore {
    fn facts_dir(&self) -> std::path::PathBuf {
        self.root().join("facts")
    }

    /// Write a durable fact as `facts/<slug>.md`, overwriting any prior content.
    pub fn save_fact(&self, fact: &Fact) -> Result<(), MemoryError> {
        let dir = self.facts_dir();
        fs::create_dir_all(&dir).map_err(|source| MemoryError::Io {
            path: dir.display().to_string(),
            source,
        })?;
        let path = dir.join(format!("{}.md", fact.slug));
        fs::write(&path, &fact.content).map_err(|source| MemoryError::Io {
            path: path.display().to_string(),
            source,
        })
    }

    /// Load a fact by slug. Missing fact → Ok(None).
    pub fn load_fact(&self, slug: &str) -> Result<Option<Fact>, MemoryError> {
        let path = self.facts_dir().join(format!("{slug}.md"));
        match fs::read_to_string(&path) {
            Ok(content) => Ok(Some(Fact {
                slug: slug.to_string(),
                content,
            })),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(MemoryError::Io {
                path: path.display().to_string(),
                source,
            }),
        }
    }

    /// List fact slugs (filenames without `.md`), sorted. Missing dir → empty.
    pub fn list_facts(&self) -> Result<Vec<String>, MemoryError> {
        let dir = self.facts_dir();
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(MemoryError::Io {
                    path: dir.display().to_string(),
                    source,
                })
            }
        };
        let mut slugs = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| MemoryError::Io {
                path: dir.display().to_string(),
                source,
            })?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(stem) = name.strip_suffix(".md") {
                slugs.push(stem.to_string());
            }
        }
        slugs.sort();
        Ok(slugs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(tag: &str) -> MemoryStore {
        let dir = std::env::temp_dir().join(format!("echo-mem-fact-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        MemoryStore::open(&dir).unwrap()
    }

    #[test]
    fn save_then_load_roundtrips() {
        let s = store("roundtrip");
        s.save_fact(&Fact {
            slug: "sarma_lives_hemel".into(),
            content: "Sarma lives in Hemel Hempstead.".into(),
        })
        .unwrap();
        let f = s.load_fact("sarma_lives_hemel").unwrap().unwrap();
        assert_eq!(f.content, "Sarma lives in Hemel Hempstead.");
    }

    #[test]
    fn load_missing_fact_is_none() {
        let s = store("none");
        assert!(s.load_fact("nope").unwrap().is_none());
    }

    #[test]
    fn list_facts_sorted() {
        let s = store("list");
        s.save_fact(&Fact { slug: "b_fact".into(), content: "b".into() }).unwrap();
        s.save_fact(&Fact { slug: "a_fact".into(), content: "a".into() }).unwrap();
        assert_eq!(s.list_facts().unwrap(), vec!["a_fact", "b_fact"]);
    }
}
