use crate::types::Capability;

/// Inputs to the per-brain scoring function (PLAN §3.2).
pub struct BrainScoreInput {
    /// Fraction (0..1) of the request's required capabilities this brain has.
    pub capability_match: f64,
    /// Fraction (0..1) of the quota window remaining.
    pub quota_remaining_pct: f64,
    /// Recent responsiveness (0..1); higher is fresher/faster.
    pub freshness: f64,
}

/// PLAN §3.2 scoring: `capability_match*100 + quota*30 + freshness*10`.
/// (User pins are handled in `pick_brain`, not here.)
pub fn score(i: &BrainScoreInput) -> f64 {
    i.capability_match * 100.0 + i.quota_remaining_pct * 30.0 + i.freshness * 10.0
}

/// Fraction of `required` capabilities present in `have`. Empty `required` → 1.0.
pub fn capability_match(have: &[Capability], required: &[Capability]) -> f64 {
    if required.is_empty() {
        return 1.0;
    }
    let met = required.iter().filter(|r| have.contains(r)).count();
    met as f64 / required.len() as f64
}

/// A brain the router may choose from.
pub struct BrainCandidate {
    pub id: String,
    pub capabilities: Vec<Capability>,
    pub quota_remaining_pct: f64,
    pub freshness: f64,
}

/// A routing request: which capabilities the task needs, and an optional pin.
#[derive(Default)]
pub struct RouteRequest {
    pub required: Vec<Capability>,
    pub pin: Option<String>,
}

/// Pick the best brain id for the request. A valid pin always wins (PLAN §3.2);
/// otherwise the highest-scoring candidate wins. Empty candidate list → None.
pub fn pick_brain(candidates: &[BrainCandidate], req: &RouteRequest) -> Option<String> {
    if let Some(pin) = &req.pin {
        if candidates.iter().any(|c| &c.id == pin) {
            return Some(pin.clone());
        }
    }
    candidates
        .iter()
        .map(|c| {
            let s = score(&BrainScoreInput {
                capability_match: capability_match(&c.capabilities, &req.required),
                quota_remaining_pct: c.quota_remaining_pct,
                freshness: c.freshness,
            });
            (c.id.clone(), s)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claude() -> BrainCandidate {
        BrainCandidate {
            id: "claude".into(),
            capabilities: vec![Capability::Code, Capability::Reason, Capability::LongContext],
            quota_remaining_pct: 0.8,
            freshness: 0.9,
        }
    }
    fn gemini() -> BrainCandidate {
        BrainCandidate {
            id: "gemini".into(),
            capabilities: vec![Capability::Reason, Capability::WebGrounding],
            quota_remaining_pct: 1.0,
            freshness: 0.5,
        }
    }

    #[test]
    fn capability_match_is_fraction() {
        let have = vec![Capability::Code, Capability::Reason];
        assert_eq!(capability_match(&have, &[Capability::Code]), 1.0);
        assert_eq!(capability_match(&have, &[Capability::Code, Capability::Vision]), 0.5);
        assert_eq!(capability_match(&have, &[]), 1.0);
    }

    #[test]
    fn pin_always_wins_when_present() {
        let cands = vec![claude(), gemini()];
        let req = RouteRequest { required: vec![Capability::WebGrounding], pin: Some("claude".into()) };
        assert_eq!(pick_brain(&cands, &req).unwrap(), "claude");
    }

    #[test]
    fn invalid_pin_is_ignored() {
        let cands = vec![claude(), gemini()];
        let req = RouteRequest { required: vec![], pin: Some("nope".into()) };
        // No pin match → falls through to scoring; both meet empty caps, so the
        // higher quota+freshness combo wins (claude: .8*30+.9*10=33; gemini: 1*30+.5*10=35).
        assert_eq!(pick_brain(&cands, &req).unwrap(), "gemini");
    }

    #[test]
    fn capability_match_dominates_score() {
        let cands = vec![claude(), gemini()];
        // Task needs web grounding: only gemini has it (match 1.0 → +100), claude 0.0.
        let req = RouteRequest { required: vec![Capability::WebGrounding], pin: None };
        assert_eq!(pick_brain(&cands, &req).unwrap(), "gemini");
    }

    #[test]
    fn empty_candidates_is_none() {
        assert!(pick_brain(&[], &RouteRequest::default()).is_none());
    }
}
