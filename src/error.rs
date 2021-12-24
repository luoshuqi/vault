use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

use log::error;
use serde_json::json;
use ws_jsonrpc::response::Error as RpcError;

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! err {
    () => {
        |e| crate::Error::new(file!(), line!(), e)
    };
    ($e:expr) => {
        crate::Error::new(file!(), line!(), $e)
    };
}

#[derive(Debug)]
pub struct Error {
    #[allow(dead_code)]
    file: &'static str,
    #[allow(dead_code)]
    line: u32,
    source: Box<dyn StdError + Send>,
}

impl Error {
    pub fn new(file: &'static str, line: u32, error: impl StdError + Send + 'static) -> Self {
        Self {
            file,
            line,
            source: Box::new(error),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.source, f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.source)
    }
}

impl From<Error> for RpcError {
    fn from(err: Error) -> Self {
        error!("{:?}", err);
        let data = json!({"kind": "InternalError"});
        RpcError::server_error(None, "Server Error", Some(data))
    }
}
