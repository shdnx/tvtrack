// We're never going to care about individual type of errors in this application.
pub type AnyError = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, AnyError>;
