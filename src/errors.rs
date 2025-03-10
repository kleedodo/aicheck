use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("response error:`{0}`")]
    ResponseError(String),
}
