use std::path::PathBuf;

use crate::types::{MinecraftVersion, ModLoader, ProjectSlug};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("toml error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("json error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("Request or response error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Response parsing error: {0}")]
    ResponseParse(serde_json::Error),

    #[error("Time parsing error: {0}")]
    TimeParse(#[from] chrono::ParseError),

    #[error("Mod id should be 8 characters: {0}")]
    ModIdTooLong(String),

    #[error("Invalid mod loader: {0}")]
    InvalidLoader(String),

    #[error("Invalid minecraft version: {0}")]
    InvalidMinecraftVersion(String),

    #[error("Not in offline cache: {0:?}")]
    CacheMiss(crate::types::ModLink),

    #[error(
        "Project {project_slug:?} has no version for game version {game_version} and mod loader {mod_loader}"
    )]
    NoMatchingVersion {
        project_slug: ProjectSlug,
        game_version: MinecraftVersion,
        mod_loader: ModLoader,
    },

    #[error("Path does not exist: {0:?}")]
    MissingPath(PathBuf),

    #[error("Failure to create path: {0:?}")]
    CreatePath(PathBuf),

    #[error("Failure to read path: {0:?}")]
    ReadPath(PathBuf),

    #[error("Failure to write path: {0:?}")]
    WritePath(PathBuf),
}

impl Error {
    pub fn invalid_loader(s: &str) -> Self {
        Error::InvalidLoader(s.to_string())
    }
}
