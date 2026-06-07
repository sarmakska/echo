//! Echo skills bus + first-party Phase 1 skills (PLAN §6).

mod files;
mod http;
mod mcp;
mod registry;
mod skill;
mod tool_call;
mod types;
mod weather;
mod websearch;

pub use files::FilesSkill;
pub use mcp::McpServer;
pub use tool_call::{parse_tool_call, ToolCall};
#[cfg(feature = "net")]
pub use http::UreqHttp;
pub use http::{FakeHttp, HttpGet};
pub use registry::SkillRegistry;
pub use skill::Skill;
pub use types::{SkillError, ToolDef};
pub use weather::{parse_open_meteo, WeatherSkill};
pub use websearch::{parse_search_results, WebSearchSkill};
