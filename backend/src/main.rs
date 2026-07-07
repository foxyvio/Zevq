use thiserror::Error;
use tokio::task;
use zevq_backend::assessment::assess_payload_json;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("assessment task failed: {0}")]
    Join(#[from] task::JoinError),
}

#[tokio::main]
async fn main() -> Result<(), PipelineError> {
    let payload = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Confident AI".to_string());
    let report_json = task::spawn_blocking(move || assess_payload_json(&payload)).await?;
    println!("{report_json}");
    Ok(())
}
