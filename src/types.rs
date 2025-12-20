use base64::Engine;

use crate::error::{Error, Result};

pub use moddb::ModDB;
pub use version::MinecraftVersion;

mod moddb;
mod version;

/// Enumeration of mod loader options
#[derive(
    serde::Deserialize,
    serde::Serialize,
    PartialEq,
    Eq,
    Debug,
    Clone,
    Copy,
    Hash,
    clap::ValueEnum,
    strum::EnumString,
    strum::Display,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case", parse_err_ty = Error, parse_err_fn = Error::invalid_loader)]
pub enum ModLoader {
    #[strum(to_string = "minecraft")]
    Minecraft,
    #[strum(to_string = "datapack")]
    Datapack,
    #[strum(to_string = "fabric")]
    Fabric,
    #[strum(to_string = "forge")]
    Forge,
    #[serde(rename = "neoforge")]
    #[strum(to_string = "neoforge")]
    NeoForge,
    #[strum(to_string = "quilt")]
    Quilt,
    #[strum(to_string = "babric")]
    Babric,
    #[strum(to_string = "bta-babric")]
    BtaBabric,
    #[strum(to_string = "bukkit")]
    Bukkit,
    #[strum(to_string = "bungeecord")]
    BungeeCord,
    #[strum(to_string = "canvas")]
    Canvas,
    #[strum(to_string = "folia")]
    Folia,
    #[strum(to_string = "geyser")]
    Geyser,
    #[strum(to_string = "iris")]
    Iris,
    #[strum(to_string = "java-agent")]
    JavaAgent,
    #[strum(to_string = "legacy-fabric")]
    LegacyFabric,
    #[strum(to_string = "liteloader")]
    LiteLoader,
    #[allow(clippy::enum_variant_names)]
    #[strum(to_string = "modloader")]
    ModLoader,
    #[strum(to_string = "nilloader")]
    NilLoader,
    #[strum(to_string = "optifine")]
    Optifine,
    #[strum(to_string = "ornithe")]
    Ornithe,
    #[strum(to_string = "paper")]
    Paper,
    #[strum(to_string = "purpur")]
    Purpur,
    #[strum(to_string = "rift")]
    Rift,
    #[strum(to_string = "spigot")]
    Spigot,
    #[strum(to_string = "sponge")]
    Sponge,
    #[strum(to_string = "vanilla")]
    Vanilla,
    #[strum(to_string = "velocity")]
    Velocity,
    #[strum(to_string = "waterfall")]
    Waterfall,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum ModLink {
    ProjectId(ProjectId),
    ProjectSlug(ProjectSlug),
    VersionId(VersionId),
}

impl From<ProjectId> for ModLink {
    fn from(value: ProjectId) -> Self {
        Self::ProjectId(value)
    }
}

impl From<ProjectSlug> for ModLink {
    fn from(value: ProjectSlug) -> Self {
        Self::ProjectSlug(value)
    }
}

impl From<VersionId> for ModLink {
    fn from(value: VersionId) -> Self {
        Self::VersionId(value)
    }
}

fn base64_decode_id(value: &str) -> Result<u64> {
    let mut vec = base64::prelude::BASE64_STANDARD_NO_PAD.decode(value)?;
    if vec.len() > 8 {
        return Err(Error::ModIdTooLong(value.into()));
    }
    vec.resize(size_of::<u64>(), 0);
    Ok(u64::from_le_bytes(
        vec.split_at(size_of::<u64>()).0.try_into().unwrap(),
    ))
}

fn base64_encode_id(value: u64) -> String {
    base64::prelude::BASE64_STANDARD_NO_PAD.encode(value.to_le_bytes())[0..8].to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "&str", into = "String")]
pub struct ProjectId(u64);

impl ProjectId {
    pub fn inner(self) -> u64 {
        self.0
    }
}

impl TryFrom<&str> for ProjectId {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        Ok(Self(base64_decode_id(value)?))
    }
}

impl From<ProjectId> for String {
    fn from(value: ProjectId) -> Self {
        value.to_string()
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", base64_encode_id(self.0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(from = "&str", into = "String")]
pub struct ProjectSlug(String);

impl ProjectSlug {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn inner(&self) -> &String {
        &self.0
    }
}

impl From<&str> for ProjectSlug {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for ProjectSlug {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ProjectSlug> for String {
    fn from(value: ProjectSlug) -> Self {
        value.0
    }
}

impl std::fmt::Display for ProjectSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "&str", into = "String")]
pub struct VersionId(u64);

impl VersionId {
    pub fn inner(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for VersionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", base64_encode_id(self.0))
    }
}

impl TryFrom<&str> for VersionId {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        Ok(Self(base64_decode_id(value)?))
    }
}

impl From<VersionId> for String {
    fn from(value: VersionId) -> Self {
        value.to_string()
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ModProject {
    pub project_id: ProjectId,
    pub name: String,
    pub slug: ProjectSlug,
    pub version_ids: Vec<VersionId>,
    pub game_versions: Vec<MinecraftVersion>,
    pub loaders: Vec<ModLoader>,
}

mod serde_naive_date_time {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

    pub fn serialize<S: Serializer>(
        time: &NaiveDateTime,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        DateTime::<Utc>::from_naive_utc_and_offset(*time, Utc)
            .to_rfc3339()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<NaiveDateTime, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        Ok(DateTime::parse_from_rfc3339(&time)
            .map_err(D::Error::custom)?
            .naive_utc())
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ModVersion {
    pub project_id: ProjectId,
    pub version_id: VersionId,
    pub name: String,
    pub game_versions: Vec<MinecraftVersion>,
    pub loaders: Vec<ModLoader>,
    pub files: Vec<ModFile>,
    pub dependencies: Vec<ModLink>,
    #[serde(with = "serde_naive_date_time")]
    pub date_published: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ModFile {
    pub url: String,
    pub name: String,
}
