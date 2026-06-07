use echo_brain::{Brain, Context, Prompt};
use echo_skills::{parse_tool_call, SkillRegistry};

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("brain error: {0}")]
    Brain(String),
    #[error("skill error: {0}")]
    Skill(String),
    #[error("agent exceeded {0} steps without a final answer")]
    MaxSteps(usize),
}

/// In-process agentic loop (ARCHITECTURE step 7): ask the brain; if its reply is
/// a tool call, dispatch it to the skill registry and feed the result back; loop
/// until the brain returns a plain answer or `max_steps` is hit.
pub struct AgentLoop<'a, B: Brain> {
    brain: &'a B,
    skills: &'a SkillRegistry,
    max_steps: usize,
}

impl<'a, B: Brain> AgentLoop<'a, B> {
    pub fn new(brain: &'a B, skills: &'a SkillRegistry) -> Self {
        Self { brain, skills, max_steps: 6 }
    }

    pub fn with_max_steps(mut self, n: usize) -> Self {
        self.max_steps = n;
        self
    }

    /// Run the loop. `system` is the recalled context; `user` the utterance.
    pub fn run(&self, system: &str, user: &str) -> Result<String, AgentError> {
        let mut conversation = user.to_string();
        for _ in 0..self.max_steps {
            let response = self
                .brain
                .ask(&Prompt::new(&conversation), &Context { system: system.to_string() })
                .map_err(|e| AgentError::Brain(e.to_string()))?;

            match parse_tool_call(&response.text) {
                Some(call) => {
                    let result = self
                        .skills
                        .call(&call.tool, &call.args)
                        .map_err(|e| AgentError::Skill(e.to_string()))?;
                    conversation = format!(
                        "{conversation}\n\n[tool {} result]: {}\n\nUse this to answer.",
                        call.tool, result
                    );
                }
                None => return Ok(response.text),
            }
        }
        Err(AgentError::MaxSteps(self.max_steps))
    }
}
