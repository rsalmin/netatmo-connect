use reqwest;
use confy;
use url;

#[derive(Debug)]
pub struct Error {
  pub msg : String,
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
