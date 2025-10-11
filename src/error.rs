pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    #[allow(dead_code)]
    IO(std::io::Error),
    #[allow(dead_code)]
    TomlParse(toml::de::Error),
    #[allow(dead_code)]
    JsonParse(serde_json::Error),
    #[allow(dead_code)]
    Request(reqwest::Error),
    #[allow(dead_code)]
    ResponseEmpty { url: String },
    #[allow(dead_code)]
    InvalidLoader(String),
}

impl Error {
    pub fn invalid_loader(s: &str) -> Self {
        Error::InvalidLoader(s.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IO(value)
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::TomlParse(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Request(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::JsonParse(value)
    }
}
