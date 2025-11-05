#[derive(Debug, thiserror::Error)]
#[error("Wat parsing failed: {0}")]
pub struct WatParseError(pub String);

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error("Wasmparser, reason: {0}")]
    Wasmparser(String),
    #[error("ContextCreation1, reason: {0}")]
    ContextCreation1(String),
    #[error("ContextCreation2 @ {0}")]
    ContextCreation2(Box<str>),
    #[error("FindTextOffset1, reason: {0}")]
    FindTextOffset1(String),
    #[error("FindTextOffset2 @ {0}")]
    FindTextOffset2(Box<str>),
    #[error("No code section found.")]
    NoCodeSection,
}
