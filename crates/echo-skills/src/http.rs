use crate::types::SkillError;

/// Minimal HTTP GET boundary so network skills are testable with fakes.
pub trait HttpGet {
    fn get(&self, url: &str) -> Result<String, SkillError>;
}

/// Test double: returns a canned body (or a canned error) for any URL.
pub struct FakeHttp {
    body: Result<String, String>,
}
impl FakeHttp {
    pub fn ok(body: impl Into<String>) -> Self {
        Self { body: Ok(body.into()) }
    }
    pub fn failing(msg: impl Into<String>) -> Self {
        Self { body: Err(msg.into()) }
    }
}
impl HttpGet for FakeHttp {
    fn get(&self, _url: &str) -> Result<String, SkillError> {
        self.body.clone().map_err(SkillError::Http)
    }
}

/// Real blocking HTTP adapter. Compiled only with `--features net`.
#[cfg(feature = "net")]
pub struct UreqHttp;

#[cfg(feature = "net")]
impl HttpGet for UreqHttp {
    fn get(&self, url: &str) -> Result<String, SkillError> {
        ureq::get(url)
            .call()
            .map_err(|e| SkillError::Http(e.to_string()))?
            .into_string()
            .map_err(|e| SkillError::Http(e.to_string()))
    }
}
