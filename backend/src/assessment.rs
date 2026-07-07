use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const ENGINE_VERSION: &str = "0.3.0";
pub const SCHEMA_VERSION: &str = "zevq.assessment.v1";

#[derive(Debug, Error)]
pub enum AssessmentError {
    #[error("invalid startup profile: {0}")]
    Validation(String),
    #[error("failed to serialize assessment report: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartupProfile {
    pub name: String,
    pub domain: String,
    pub claimed_accuracy: f64,
    pub eval_pass_rate: f64,
    pub adversarial_resilience: f64,
    pub data_governance: f64,
    pub incidents_last_90_days: u32,
    pub p95_latency_ms: f64,
    pub has_human_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScenarioStatus {
    Pass,
    Watch,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MathScenario {
    pub id: String,
    pub category: String,
    pub weight: f64,
    pub observed: f64,
    pub required: f64,
    pub margin: f64,
    pub status: ScenarioStatus,
    pub proof: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Rating {
    Aaa,
    Aa,
    A,
    Bbb,
    Bb,
    B,
    C,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssessmentReport {
    pub schema_version: String,
    pub engine_version: String,
    pub startup: StartupProfile,
    pub rating: Rating,
    pub overall_score: f64,
    pub reliability_score: f64,
    pub safety_score: f64,
    pub governance_score: f64,
    pub scenarios: Vec<MathScenario>,
    pub remediation_plan: Vec<String>,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorReport {
    pub schema_version: String,
    pub engine_version: String,
    pub verdict: String,
    pub error: String,
}

pub fn parse_or_default_profile(payload: &str) -> StartupProfile {
    parse_profile(payload).unwrap_or_else(|_| default_profile(payload))
}

pub fn parse_profile(payload: &str) -> Result<StartupProfile, AssessmentError> {
    let profile = if payload.trim_start().starts_with('{') {
        serde_json::from_str::<StartupProfile>(payload).map_err(|error| {
            AssessmentError::Validation(format!("profile JSON is malformed: {error}"))
        })?
    } else {
        default_profile(payload)
    };

    validate_profile(&profile)?;
    Ok(profile)
}

pub fn assess_startup(profile: StartupProfile) -> AssessmentReport {
    let profile = normalize_profile(profile);
    let reliability_score =
        bounded_percent(100.0 * profile.eval_pass_rate.min(profile.claimed_accuracy));
    let incident_penalty = (profile.incidents_last_90_days as f64 * 7.5).min(45.0);
    let override_bonus = if profile.has_human_override {
        8.0
    } else {
        -8.0
    };
    let safety_score = bounded_percent(
        (profile.adversarial_resilience * 100.0) - incident_penalty + override_bonus,
    );
    let latency_penalty = ((profile.p95_latency_ms - 250.0).max(0.0) / 20.0).min(20.0);
    let governance_score =
        bounded_percent((profile.data_governance * 100.0) + override_bonus - latency_penalty);
    let overall_score =
        round2((0.42 * reliability_score) + (0.38 * safety_score) + (0.20 * governance_score));
    let rating = rating_for(overall_score);
    let scenarios = build_scenarios(&profile, reliability_score, safety_score, governance_score);
    let remediation_plan = remediation_plan(&scenarios);
    let verdict = verdict_for(&rating, overall_score, &scenarios);

    AssessmentReport {
        schema_version: SCHEMA_VERSION.to_string(),
        engine_version: ENGINE_VERSION.to_string(),
        startup: profile,
        rating,
        overall_score,
        reliability_score,
        safety_score,
        governance_score,
        scenarios,
        remediation_plan,
        verdict,
    }
}

pub fn assessment_to_json(report: &AssessmentReport) -> Result<String, AssessmentError> {
    Ok(serde_json::to_string_pretty(report)?)
}

pub fn assess_payload_json(payload: &str) -> String {
    match parse_profile(payload) {
        Ok(profile) => assessment_to_json(&assess_startup(profile)).unwrap_or_else(error_json),
        Err(error) => error_json(error),
    }
}

fn default_profile(payload: &str) -> StartupProfile {
    StartupProfile {
        name: if payload.trim().is_empty() {
            "Confident AI Benchmark Target".to_string()
        } else {
            payload.trim().chars().take(80).collect()
        },
        domain: "general_ai_startup".to_string(),
        claimed_accuracy: 0.94,
        eval_pass_rate: 0.87,
        adversarial_resilience: 0.71,
        data_governance: 0.68,
        incidents_last_90_days: 2,
        p95_latency_ms: 420.0,
        has_human_override: false,
    }
}

fn validate_profile(profile: &StartupProfile) -> Result<(), AssessmentError> {
    if profile.name.trim().is_empty() {
        return Err(AssessmentError::Validation(
            "startup name is required".to_string(),
        ));
    }
    for (field, value) in [
        ("claimed_accuracy", profile.claimed_accuracy),
        ("eval_pass_rate", profile.eval_pass_rate),
        ("adversarial_resilience", profile.adversarial_resilience),
        ("data_governance", profile.data_governance),
    ] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(AssessmentError::Validation(format!(
                "{field} must be between 0.0 and 1.0"
            )));
        }
    }
    if !profile.p95_latency_ms.is_finite() || profile.p95_latency_ms < 0.0 {
        return Err(AssessmentError::Validation(
            "p95_latency_ms must be a non-negative finite number".to_string(),
        ));
    }
    Ok(())
}

fn normalize_profile(mut profile: StartupProfile) -> StartupProfile {
    profile.name = profile.name.trim().chars().take(80).collect();
    profile.domain = profile.domain.trim().chars().take(80).collect();
    profile
}

fn build_scenarios(
    profile: &StartupProfile,
    reliability_score: f64,
    safety_score: f64,
    governance_score: f64,
) -> Vec<MathScenario> {
    vec![
        scenario("MATH-RELIABILITY-001", "claim_vs_eval_reliability", 0.42, reliability_score, 90.0, "min(claimed_accuracy, eval_pass_rate) * 100 must clear 90 for enterprise certification.", "Raise independent eval pass rate and reduce unsupported accuracy claims."),
        scenario("MATH-SAFETY-002", "adversarial_resilience_minus_incident_penalty", 0.38, safety_score, 85.0, "adversarial_resilience*100 - incidents*7.5 + human_override_bonus must clear 85.", "Improve adversarial test pass rate, reduce recent incidents, and add human override."),
        scenario("MATH-GOVERNANCE-003", "governance_latency_and_human_control", 0.20, governance_score, 80.0, "data_governance*100 + override_bonus - latency_penalty must clear 80.", "Improve data governance, incident response, latency controls, and operator override."),
        max_scenario("MATH-CLAIM-GAP-004", "overclaim_gap_guardrail", 0.15, round2((profile.eval_pass_rate - profile.claimed_accuracy).abs() * 100.0), 5.0, "absolute(claimed_accuracy - eval_pass_rate) must stay within 5 percentage points.", "Align marketing claims with measured evaluation evidence."),
    ]
}

fn scenario(
    id: &str,
    category: &str,
    weight: f64,
    observed: f64,
    required: f64,
    proof: &str,
    remediation: &str,
) -> MathScenario {
    let margin = round2(observed - required);
    let status = if observed >= required {
        ScenarioStatus::Pass
    } else if observed + 7.5 >= required {
        ScenarioStatus::Watch
    } else {
        ScenarioStatus::Fail
    };

    MathScenario {
        id: id.to_string(),
        category: category.to_string(),
        weight,
        observed: round2(observed),
        required,
        margin,
        status,
        proof: proof.to_string(),
        remediation: remediation.to_string(),
    }
}

fn max_scenario(
    id: &str,
    category: &str,
    weight: f64,
    observed: f64,
    maximum: f64,
    proof: &str,
    remediation: &str,
) -> MathScenario {
    let margin = round2(maximum - observed);
    let status = if observed <= maximum {
        ScenarioStatus::Pass
    } else if observed <= maximum + 7.5 {
        ScenarioStatus::Watch
    } else {
        ScenarioStatus::Fail
    };

    MathScenario {
        id: id.to_string(),
        category: category.to_string(),
        weight,
        observed: round2(observed),
        required: maximum,
        margin,
        status,
        proof: proof.to_string(),
        remediation: remediation.to_string(),
    }
}

fn remediation_plan(scenarios: &[MathScenario]) -> Vec<String> {
    scenarios
        .iter()
        .filter(|scenario| scenario.status != ScenarioStatus::Pass)
        .map(|scenario| format!("{}: {}", scenario.id, scenario.remediation))
        .collect()
}

fn rating_for(score: f64) -> Rating {
    match score {
        score if score >= 95.0 => Rating::Aaa,
        score if score >= 90.0 => Rating::Aa,
        score if score >= 82.0 => Rating::A,
        score if score >= 74.0 => Rating::Bbb,
        score if score >= 64.0 => Rating::Bb,
        score if score >= 50.0 => Rating::B,
        _ => Rating::C,
    }
}

fn verdict_for(rating: &Rating, score: f64, scenarios: &[MathScenario]) -> String {
    let fail_count = scenarios
        .iter()
        .filter(|scenario| scenario.status == ScenarioStatus::Fail)
        .count();
    match (rating, fail_count) {
        (Rating::Aaa | Rating::Aa, 0) => {
            format!("CERTIFY: mathematical score {score} clears premium enterprise threshold.")
        }
        (Rating::A | Rating::Bbb, 0 | 1) => {
            format!("CONDITIONAL: score {score} is investable but needs remediation evidence.")
        }
        _ => {
            format!("REJECT_FOR_NOW: score {score} has {fail_count} hard mathematical failure(s).")
        }
    }
}

fn error_json(error: AssessmentError) -> String {
    let report = ErrorReport {
        schema_version: SCHEMA_VERSION.to_string(),
        engine_version: ENGINE_VERSION.to_string(),
        verdict: "REJECT_FOR_NOW".to_string(),
        error: error.to_string(),
    };
    serde_json::to_string_pretty(&report).unwrap_or_else(|_| {
        "{\"verdict\":\"REJECT_FOR_NOW\",\"error\":\"fatal serialization failure\"}".to_string()
    })
}

fn bounded_percent(value: f64) -> f64 {
    round2(value.clamp(0.0, 100.0))
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_startup_gets_rejected_with_failures() {
        let report = assess_startup(parse_or_default_profile("Confident AI"));
        assert!(matches!(report.rating, Rating::Bb | Rating::B | Rating::C));
        assert!(report.verdict.starts_with("REJECT_FOR_NOW"));
        assert!(report
            .scenarios
            .iter()
            .any(|scenario| scenario.status == ScenarioStatus::Fail));
        assert!(!report.remediation_plan.is_empty());
    }

    #[test]
    fn strong_startup_gets_certified() {
        let report = assess_startup(StartupProfile {
            name: "Elite Safety AI".to_string(),
            domain: "regulated_ai".to_string(),
            claimed_accuracy: 0.97,
            eval_pass_rate: 0.96,
            adversarial_resilience: 0.96,
            data_governance: 0.95,
            incidents_last_90_days: 0,
            p95_latency_ms: 180.0,
            has_human_override: true,
        });
        assert!(matches!(report.rating, Rating::Aaa | Rating::Aa));
        assert!(report.verdict.starts_with("CERTIFY"));
        assert!(report.remediation_plan.is_empty());
    }

    #[test]
    fn invalid_json_returns_error_report() {
        let json = assess_payload_json("{bad json");
        assert!(json.contains("REJECT_FOR_NOW"));
        assert!(json.contains("profile JSON is malformed"));
    }

    #[test]
    fn invalid_score_is_rejected() {
        let payload = r#"{"name":"Bad AI","domain":"ai","claimed_accuracy":1.2,"eval_pass_rate":0.9,"adversarial_resilience":0.8,"data_governance":0.8,"incidents_last_90_days":0,"p95_latency_ms":100.0,"has_human_override":true}"#;
        let json = assess_payload_json(payload);
        assert!(json.contains("claimed_accuracy must be between 0.0 and 1.0"));
    }
}
