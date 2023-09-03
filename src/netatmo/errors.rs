use reqwest;
use confy;
use url;
use tokio::task::JoinError;

#[derive(Debug)]
pub struct Error {
    msg : String,
}

impl Error {
pub fn new(msg : &str) -> Error {
    Error { msg : String::from(msg) }
}
}

impl From<Error> for String {
  fn from(e : Error) -> String {
    e.msg
  }
}

impl From<reqwest::Error> for Error {
  fn from(e : reqwest::Error) -> Error {
    Error { msg : format!("{:?}", e) }
  }
}

impl From<url::ParseError> for Error {
  fn from(e : url::ParseError) -> Error {
    Error { msg : format!("{:?}", e) }
  }
}

impl From<std::io::Error> for Error {
  fn from(e : std::io::Error) -> Error {
    Error { msg : format!("{:?}", e) }
  }
}

impl From<JoinError> for Error {
  fn from(e : JoinError) -> Error {
    Error { msg : format!("{:?}", e) }
  }
}

impl From<String> for Error {
  fn from(e : String) -> Error {
    Error { msg : e }
  }
}

impl From<confy::ConfyError> for Error {
  fn from(e : confy::ConfyError) -> Error {
    Error { msg : format!("{:?}", e) }
  }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
  fn from(e : std::sync::PoisonError<T>) -> Error {
    Error { msg : format!("{:?}", e) }
  }
}
