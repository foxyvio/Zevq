use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssessmentError {
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
    pub startup: StartupProfile,
    pub rating: Rating,
    pub overall_score: f64,
    pub reliability_score: f64,
    pub safety_score: f64,
    pub governance_score: f64,
    pub scenarios: Vec<MathScenario>,
    pub verdict: String,
}

pub fn parse_or_default_profile(payload: &str) -> StartupProfile {
    serde_json::from_str::<StartupProfile>(payload).unwrap_or_else(|_| StartupProfile {
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
    })
}

pub fn assess_startup(profile: StartupProfile) -> AssessmentReport {
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
    let verdict = verdict_for(&rating, overall_score, &scenarios);

    AssessmentReport {
        startup: profile,
        rating,
        overall_score,
        reliability_score,
        safety_score,
        governance_score,
        scenarios,
        verdict,
    }
}

pub fn assessment_to_json(report: &AssessmentReport) -> Result<String, AssessmentError> {
    Ok(serde_json::to_string_pretty(report)?)
}

fn build_scenarios(
    profile: &StartupProfile,
    reliability_score: f64,
    safety_score: f64,
    governance_score: f64,
) -> Vec<MathScenario> {
    vec![
        scenario(
            "MATH-RELIABILITY-001",
            "claim_vs_eval_reliability",
            0.42,
            reliability_score,
            90.0,
            "min(claimed_accuracy, eval_pass_rate) * 100 must clear 90 for enterprise certification.",
        ),
        scenario(
            "MATH-SAFETY-002",
            "adversarial_resilience_minus_incident_penalty",
            0.38,
            safety_score,
            85.0,
            "adversarial_resilience*100 - incidents*7.5 + human_override_bonus must clear 85.",
        ),
        scenario(
            "MATH-GOVERNANCE-003",
            "governance_latency_and_human_control",
            0.20,
            governance_score,
            80.0,
            "data_governance*100 + override_bonus - latency_penalty must clear 80.",
        ),
        max_scenario(
            "MATH-CLAIM-GAP-004",
            "overclaim_gap_guardrail",
            0.15,
            round2((profile.eval_pass_rate - profile.claimed_accuracy).abs() * 100.0),
            5.0,
            "absolute(claimed_accuracy - eval_pass_rate) must stay within 5 percentage points.",
        ),
    ]
}

fn scenario(
    id: &str,
    category: &str,
    weight: f64,
    observed: f64,
    required: f64,
    proof: &str,
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
    }
}

fn max_scenario(
    id: &str,
    category: &str,
    weight: f64,
    observed: f64,
    maximum: f64,
    proof: &str,
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
    }
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
    }
}
