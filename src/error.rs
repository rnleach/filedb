/// Errors from the filedb crate.
///
/// Specific errors are given for missing or unavailable data. All errors originating in another
/// crate are boxed and passed up as a Box<dyn std::error::Error> trait object.
#[derive(Debug)]
pub enum Error {
    /// A general error originating in this crate with a message describing it.
    GeneralError(String),
    /// Any error from another library such as std or rusqlite is passed up this way.
    InternalError(Box<dyn std::error::Error>),
    /// No data for that time stamp is available, the key and time stamp are returned in the error.
    TimeStampNotAvailable(String, chrono::NaiveDateTime),
    /// There was no match for the requested key, the internal value is the requested key.
    NoMatch(String),
}

impl Error {
    /// Create a new general error with a message.
    pub fn general_error(msg: String) -> Self {
        Error::GeneralError(msg)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::GeneralError(msg) => write!(f, "{}", msg),
            Self::InternalError(err) => write!(f, "{}", err),
            Self::NoMatch(requested_key) => {
                write!(f, "No match found for key {}", requested_key)
            }
            Self::TimeStampNotAvailable(key, time_stamp) => {
                write!(
                    f,
                    "No data available for key {} and time stamp {}",
                    key, time_stamp
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::InternalError(err) => Some(err.as_ref()),
            _ => None,
        }
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InternalError(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::InternalError(err.into())
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Self::InternalError(err.into())
    }
}

/* Ideal implementation below, but this might need to wait for specialization to land on stable.
impl<E> From<E> for Error
where Box<dyn std::error::Error>: From<E>
{
    fn from(err: E) -> Self {
        Self::InternalError(err.into())
    }
}
*/
