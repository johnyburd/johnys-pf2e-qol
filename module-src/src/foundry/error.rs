use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0:?}")]
    JsValue(JsValue),
    #[error("{0}")]
    Custom(String),
    #[error("{0}: {1}")]
    Context(String, Box<Self>),
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_owned())
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Self::JsValue(value)
    }
}

impl Error {
    pub fn ctx(self, msg: &str) -> Self {
        Self::Context(msg.to_owned(), Box::new(self))
    }
}

pub trait ContextExt<T> {
    fn ctx(self, msg: &str) -> Result<T, Error>;
}

impl<T> ContextExt<T> for Result<T, Error> {
    fn ctx(self, msg: &str) -> Result<T, Error> {
        self.map_err(|e| e.ctx(msg))
    }
}



impl<T> ContextExt<T> for Option<T> {
    fn ctx(self, msg: &str) -> Result<T, Error> {
        self.ok_or_else(|| Error::Custom(msg.to_owned()))
    }
}
