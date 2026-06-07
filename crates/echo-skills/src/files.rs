use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::skill::Skill;
use crate::types::{SkillError, ToolDef};

/// Local file access rooted at `root`. Paths are resolved under root and any
/// attempt to escape (via `..` or absolute paths) is rejected (PLAN §6.2).
pub struct FilesSkill {
    root: PathBuf,
}

impl FilesSkill {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf() }
    }

    fn resolve(&self, rel: &str) -> Result<PathBuf, SkillError> {
        let candidate = Path::new(rel);
        if candidate.is_absolute() || rel.split('/').any(|c| c == "..") {
            return Err(SkillError::BadArgs(format!("path escapes sandbox: {rel}")));
        }
        Ok(self.root.join(candidate))
    }

    fn arg_str(args: &Value, key: &str) -> Result<String, SkillError> {
        args.get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| SkillError::BadArgs(format!("missing string arg `{key}`")))
    }
}

impl Skill for FilesSkill {
    fn name(&self) -> &str {
        "files-local"
    }

    fn list_tools(&self) -> Vec<ToolDef> {
        vec![
            ToolDef::new("read_file", "Read a UTF-8 file under the sandbox root"),
            ToolDef::new("write_file", "Write a UTF-8 file under the sandbox root"),
            ToolDef::new("list_dir", "List entries of a directory under the root"),
        ]
    }

    fn call_tool(&self, tool: &str, args: &Value) -> Result<Value, SkillError> {
        match tool {
            "read_file" => {
                let path = self.resolve(&Self::arg_str(args, "path")?)?;
                let content =
                    fs::read_to_string(&path).map_err(|e| SkillError::Io(e.to_string()))?;
                Ok(json!({ "content": content }))
            }
            "write_file" => {
                let path = self.resolve(&Self::arg_str(args, "path")?)?;
                let content = Self::arg_str(args, "content")?;
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|e| SkillError::Io(e.to_string()))?;
                }
                fs::write(&path, content.as_bytes()).map_err(|e| SkillError::Io(e.to_string()))?;
                Ok(json!({ "ok": true }))
            }
            "list_dir" => {
                let path = self.resolve(&Self::arg_str(args, "path")?)?;
                let mut names = Vec::new();
                for entry in fs::read_dir(&path).map_err(|e| SkillError::Io(e.to_string()))? {
                    let entry = entry.map_err(|e| SkillError::Io(e.to_string()))?;
                    names.push(entry.file_name().to_string_lossy().to_string());
                }
                names.sort();
                Ok(json!({ "entries": names }))
            }
            other => Err(SkillError::UnknownTool(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(tag: &str) -> FilesSkill {
        let dir = std::env::temp_dir().join(format!("echo-skill-files-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        FilesSkill::new(dir)
    }

    #[test]
    fn write_then_read_roundtrips() {
        let s = skill("rw");
        s.call_tool("write_file", &json!({"path": "a.txt", "content": "hi"})).unwrap();
        let out = s.call_tool("read_file", &json!({"path": "a.txt"})).unwrap();
        assert_eq!(out["content"], "hi");
    }

    #[test]
    fn list_dir_returns_sorted_entries() {
        let s = skill("ls");
        s.call_tool("write_file", &json!({"path": "b.txt", "content": "b"})).unwrap();
        s.call_tool("write_file", &json!({"path": "a.txt", "content": "a"})).unwrap();
        let out = s.call_tool("list_dir", &json!({"path": "."})).unwrap();
        assert_eq!(out["entries"], json!(["a.txt", "b.txt"]));
    }

    #[test]
    fn rejects_path_escape() {
        let s = skill("escape");
        let err = s.call_tool("read_file", &json!({"path": "../secret"})).unwrap_err();
        assert!(matches!(err, SkillError::BadArgs(_)));
    }

    #[test]
    fn missing_arg_errors() {
        let s = skill("noarg");
        assert!(matches!(s.call_tool("read_file", &json!({})).unwrap_err(), SkillError::BadArgs(_)));
    }
}
