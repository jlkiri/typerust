use thiserror::Error;

pub type Result<T> = std::result::Result<T, SandboxError>;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("out of memory error")]
    OOM,
    #[error("internal error: {0}")]
    Internal(String),
    #[error("timeout error")]
    Timeout,
}

impl From<anyhow::Error> for SandboxError {
    fn from(err: anyhow::Error) -> Self {
        let message = err.to_string();
        err.downcast::<SandboxError>()
            .unwrap_or(SandboxError::Internal(message))
    }
}
