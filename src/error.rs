use std::string::FromUtf8Error;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    Io(std::io::Error),
    UnknownLang(String),
    MetadataConversion(FromUtf8Error),
    Custom(String),
    Serde(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::Custom(s)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Error {
        Error::MetadataConversion(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Serde(e)
    }
}
