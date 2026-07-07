use thiserror::Error;
use tokio::task;
use zevq_backend::{
    assess_startup,
    assessment::{assessment_to_json, AssessmentError},
    parse_or_default_profile,
};

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("assessment task failed: {0}")]
    Join(#[from] task::JoinError),
    #[error(transparent)]
    Assessment(#[from] AssessmentError),
}

#[tokio::main]
async fn main() -> Result<(), PipelineError> {
    let payload = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Confident AI".to_string());
    let report_json = task::spawn_blocking(move || {
        let profile = parse_or_default_profile(&payload);
        assessment_to_json(&assess_startup(profile))
    })
    .await??;
    println!("{report_json}");
    Ok(())
}
