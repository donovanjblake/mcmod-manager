pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConfigParse(toml::de::Error),
    ConfigKey(String),
    ConfigType { key: String, message: String },
    ProjectNotResolved { project: String, key: String },
    Request(reqwest::Error),
    ResponseParse(serde_json::Error),
    ResponseKey(String),
    ResponseType { key: String, message: String },
    ResponseEmpty { url: String },
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::ConfigParse(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Request(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::ResponseParse(value)
    }
}
