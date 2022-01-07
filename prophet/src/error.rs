/// This type represents all possible errors that can occur when analyzing
/// a microservice project
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Could not clone repository: {0}")]
    CloneRepo(String),
    #[error("Encountered an IO error: {0}")]
    Io(String),
    #[error("Could not create an AppData from the provided ReSSA: {0}")]
    AppData(String),
    #[error("Could not create bounded context")]
    BoundedContext(#[from] prophet_bounded_context::Error),
}

macro_rules! error_from_impl {
    ( $( $error_type:path: $variant:ident ),+ ) => {
        $(
            impl From<$error_type> for Error {
                fn from(error: $error_type) -> Self {
                    Error::$variant(error.to_string())
                }
            }
        )*
    };
}

error_from_impl!(git2::Error: CloneRepo, std::io::Error: Io);
