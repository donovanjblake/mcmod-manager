pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConfigParse(toml::de::Error),
    ConfigKey(String),
    ConfigType { key: String, message: String },
    ProjectNotResolved { project: String, key: String },
}
