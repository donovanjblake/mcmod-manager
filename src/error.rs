use toml;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConfigParseError(toml::de::Error),
    ConfigKeyError(String),
    ConfigTypeError { key: String, message: String },
    ProjectNotResolvedError { project: String, key: String },
}
