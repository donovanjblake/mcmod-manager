use std::collections::HashMap;

use crate::error::{Error, Result};

/// Enumeration of mod loader options
#[derive(
    serde::Deserialize,
    serde::Serialize,
    PartialEq,
    Eq,
    Debug,
    Clone,
    Copy,
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

/// Minecraft version structure
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(try_from = "String", into = "String")]
pub enum MinecraftVersion {
    Release {
        /// Major version number
        major: u8,
        /// Minor version number
        minor: u8,
        /// Patch version number
        patch: Option<u8>,
        /// Release suffix
        suffix: MinecraftReleaseSuffix,
    },
    Snapshot {
        /// The year the snapshot was published
        year: u8,
        /// The week the snapshot was published
        week: u8,
        /// A unique identifier to distinguish between multiple snapshots in a week
        ident: Option<u8>,
    },
    Beta {
        /// Minor version number
        major: u8,
        /// Minor version number
        minor: u8,
        /// Patch version number
        patch: Option<u8>,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(try_from = "String", into = "String")]
pub enum MinecraftReleaseSuffix {
    /// No release suffix
    None,
    /// Pre release number
    PreRelease(u8),
    /// Release candidate number
    Candidate(u8),
}

impl std::fmt::Display for MinecraftVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MinecraftVersion::Release {
                major,
                minor,
                patch,
                suffix,
            } => write!(
                f,
                "{}.{}.{}{}",
                major,
                minor,
                patch.map_or_else(|| String::from("x"), |x| x.to_string()),
                suffix
            ),
            MinecraftVersion::Snapshot { year, week, ident } => {
                write!(
                    f,
                    "{}w{}{}",
                    year,
                    week,
                    ident.map_or_else(
                        || String::from(""),
                        |x| String::from_utf8(vec![x]).expect("Invalid utf-8 in snapshot")
                    )
                )
            }
            MinecraftVersion::Beta {
                major,
                minor,
                patch,
            } => {
                write!(
                    f,
                    "b{}.{}.{}",
                    major,
                    minor,
                    patch.map_or_else(|| String::from("x"), |x| x.to_string())
                )
            }
        }
    }
}

impl std::fmt::Display for MinecraftReleaseSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MinecraftReleaseSuffix::None => write!(f, ""),
            MinecraftReleaseSuffix::PreRelease(x) => write!(f, "-pre{x}"),
            MinecraftReleaseSuffix::Candidate(x) => write!(f, "-rc{x}"),
        }
    }
}

impl From<MinecraftReleaseSuffix> for String {
    fn from(value: MinecraftReleaseSuffix) -> Self {
        format!("{}", value)
    }
}

impl TryFrom<String> for MinecraftReleaseSuffix {
    type Error = Error;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(MinecraftReleaseSuffix::None);
        }
        let kind = value
            .get(0..value.len() - 1)
            .ok_or_else(|| Error::InvalidMinecraftVersion(value.clone()))?;
        let number = value
            .get(value.len() - 1..)
            .and_then(|x| x.parse::<u8>().ok())
            .ok_or_else(|| Error::InvalidMinecraftVersion(value.clone()))?;
        match kind {
            "pre" => Ok(MinecraftReleaseSuffix::PreRelease(number)),
            "rc" => Ok(MinecraftReleaseSuffix::Candidate(number)),
            _ => Err(Error::InvalidMinecraftVersion(value.clone())),
        }
    }
}

impl From<MinecraftVersion> for String {
    fn from(value: MinecraftVersion) -> Self {
        format!("{}", value)
    }
}

impl TryFrom<String> for MinecraftVersion {
    type Error = Error;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let parts: Vec<_> = value.split(&['.', '-']).collect();
        let parse_u8 = |s: &str| -> Result<u8> {
            s.parse::<u8>()
                .map_err(|_| Error::InvalidMinecraftVersion(value.to_string()))
        };
        match parts.len() {
            1 => {
                let parts: Vec<_> = value.split("w").collect();
                if parts.len() != 2 {
                    return Err(Error::InvalidMinecraftVersion(value.to_string()));
                }
                let year = parse_u8(parts[0])?;
                let week = parse_u8(parts[1].get(0..2).expect(""))?;
                let ident = parts[1]
                    .matches(|x: char| x.is_ascii_alphabetic())
                    .next()
                    .map(|x| x.as_bytes()[0]);
                Ok(MinecraftVersion::Snapshot { year, week, ident })
            }
            2 | 3 if value.starts_with('b') => {
                let (major, minor) = (parse_u8(&parts[0][1..])?, parse_u8(parts[1])?);
                let patch = match parts.get(2) {
                    Some(x) => Some(parse_u8(x)?),
                    None => None,
                };
                Ok(MinecraftVersion::Beta {
                    major,
                    minor,
                    patch,
                })
            }
            2..=4 => {
                let (major, minor) = (parse_u8(parts[0])?, parse_u8(parts[1])?);
                let (patch, suffix) = match (parts.get(2), parts.get(3)) {
                    (None, None) => (None, MinecraftReleaseSuffix::None),
                    (Some(x), None) => {
                        if value.contains('-') {
                            (None, MinecraftReleaseSuffix::try_from(x.to_string())?)
                        } else if x.eq_ignore_ascii_case("x") {
                            (None, MinecraftReleaseSuffix::None)
                        } else {
                            (Some(parse_u8(x)?), MinecraftReleaseSuffix::None)
                        }
                    }
                    (Some(x), Some(y)) => (
                        Some(parse_u8(x)?),
                        MinecraftReleaseSuffix::try_from(y.to_string())?,
                    ),
                    (None, Some(_)) => {
                        unreachable!("Can't have [3] without [2]")
                    }
                };
                Ok(MinecraftVersion::Release {
                    major,
                    minor,
                    patch,
                    suffix,
                })
            }
            _ => Err(Error::InvalidMinecraftVersion(value.to_string())),
        }
    }
}

// impl TryFrom<&str> for MinecraftVersion {
//     type Error = Error;
//     fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
//         MinecraftVersion::try_from(value.to_string())
//     }
// }

impl From<&str> for MinecraftVersion {
    fn from(value: &str) -> Self {
        MinecraftVersion::try_from(value.to_string()).expect("Invalid minecraft version")
    }
}

/// An internal database of the projects and versions collected
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ModDB {
    /// A mapping of project ids to project data
    projects: HashMap<ProjectId, ModProject>,
    /// A mapping of version ids to version data
    versions: HashMap<VersionId, ModVersion>,
    /// A mapping of project slugs to project ids
    project_slugs: HashMap<ProjectSlug, ProjectId>,
    #[serde(skip)]
    /// A map of project ids to preferred versions
    project_versions: HashMap<ProjectId, VersionId>,
}

impl ModDB {
    /// Insert a project into the database, and return the previous project at the same project_id
    pub fn add_project(&mut self, project: ModProject) -> Option<ModProject> {
        self.project_slugs
            .insert(project.slug.clone(), project.project_id.clone());
        self.projects.insert(project.project_id.clone(), project)
    }
    /// Insert a version into the database, and return the previous version at the same version_id
    pub fn add_version(&mut self, version: ModVersion) -> Option<ModVersion> {
        self.versions.insert(version.version_id.clone(), version)
    }
    pub fn contains_key(&self, mod_link: &ModLink) -> bool {
        match mod_link {
            ModLink::ProjectId(x) => self.projects.contains_key(x),
            ModLink::ProjectSlug(x) => self.project_slugs.contains_key(x),
            ModLink::VersionId(x) => self.versions.contains_key(x),
        }
    }
    pub fn remove(&mut self, mod_link: &ModLink) {
        match mod_link {
            ModLink::ProjectId(x) => {
                self.projects.remove(x);
            }
            ModLink::ProjectSlug(x) => {
                self.project_slugs.remove(x);
            }
            ModLink::VersionId(x) => {
                self.versions.remove(x);
            }
        }
    }
    /// Get a vector of all collected versions
    pub fn get_versions(&self) -> Vec<&ModVersion> {
        self.versions.values().collect()
    }
    /// Get the project of a given id
    pub fn get_project_by_id(&self, project_id: &ProjectId) -> Option<&ModProject> {
        self.projects.get(project_id)
    }
    /// Get the project of a given slug
    pub fn get_project_by_slug(&self, project_slug: &ProjectSlug) -> Option<&ModProject> {
        self.projects.get(self.project_slugs.get(project_slug)?)
    }
    /// Get the version of a given id
    pub fn get_version(&self, version_id: &VersionId) -> Option<&ModVersion> {
        self.versions.get(version_id)
    }
    /// Set the preferred version for a project, and return the previous preferred version
    pub fn set_preferred_version(
        &mut self,
        project_id: ProjectId,
        version_id: VersionId,
    ) -> Option<VersionId> {
        self.project_versions.insert(project_id, version_id)
    }
    /// Get the preferred version of a project by its id
    pub fn get_preferred_by_id(&self, project_id: &ProjectId) -> Option<&ModVersion> {
        self.project_versions
            .get(project_id)
            .and_then(|x| self.versions.get(x))
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct ProjectId(String);

impl ProjectId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn inner(&self) -> &String {
        &self.0
    }
}

impl From<String> for ProjectId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ProjectId> for String {
    fn from(value: ProjectId) -> Self {
        value.0
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct VersionId(String);

impl VersionId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn inner(&self) -> &String {
        &self.0
    }
}

impl From<String> for VersionId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<VersionId> for String {
    fn from(value: VersionId) -> Self {
        value.0
    }
}

impl std::fmt::Display for VersionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ModProject {
    pub project_id: ProjectId,
    pub name: String,
    pub slug: ProjectSlug,
    // pub version_ids: Vec<VersionId>,
    // pub game_versions: Vec<MinecraftVersion>,
    pub loaders: Vec<ModLoader>,
}

mod serde_naive_date_time {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

    pub fn serialize<S: Serializer>(
        time: &NaiveDateTime,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        DateTime::<Utc>::from_naive_utc_and_offset(time.clone(), Utc)
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
    #[cfg(test)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_full() {
        let parsed = MinecraftVersion::try_from("1.23.4")
            .expect("MinecraftVersion shall be able to parse a version string");
        assert_eq!(
            parsed,
            MinecraftVersion::Release {
                major: 1,
                minor: 23,
                patch: Some(4),
                suffix: MinecraftReleaseSuffix::None,
            }
        );
    }

    #[test]
    fn test_version_rc() {
        let parsed = MinecraftVersion::try_from("1.23.4-rc5").expect(
            "MinecraftVersion shall be able to parse a version string of a release candidate",
        );
        assert_eq!(
            parsed,
            MinecraftVersion::Release {
                major: 1,
                minor: 23,
                patch: Some(4),
                suffix: MinecraftReleaseSuffix::Candidate(5),
            }
        );
    }

    #[test]
    fn test_version_patch_x() {
        let parsed = MinecraftVersion::try_from("1.23.x").expect("MinecraftVersion shall be able to parse a version string where the patch version is 'x'");
        assert_eq!(
            parsed,
            MinecraftVersion::Release {
                major: 1,
                minor: 23,
                patch: None,
                suffix: MinecraftReleaseSuffix::None,
            }
        );
    }

    #[test]
    fn test_version_patch_none() {
        let parsed = MinecraftVersion::try_from("1.23").expect("MinecraftVersion shall be able to parse a version string where the patch version is not given");
        assert_eq!(
            parsed,
            MinecraftVersion::Release {
                major: 1,
                minor: 23,
                patch: None,
                suffix: MinecraftReleaseSuffix::None,
            }
        );
    }

    #[test]
    fn test_version_pre() {
        let parsed = MinecraftVersion::try_from("1.23.4-pre5").expect("MinecraftVersion shall be able to parse a version string where the patch version is not given");
        assert_eq!(
            parsed,
            MinecraftVersion::Release {
                major: 1,
                minor: 23,
                patch: Some(4),
                suffix: MinecraftReleaseSuffix::PreRelease(5),
            }
        );
    }

    #[test]
    fn test_version_snapshot_twodigit() {
        let parsed = MinecraftVersion::try_from("12w34a")
            .expect("MinecraftVersion shal lbe able to parse a snapshot version string");
        assert_eq!(
            parsed,
            MinecraftVersion::Snapshot {
                year: 12,
                week: 34,
                ident: Some("a".as_bytes()[0])
            }
        )
    }

    #[test]
    fn test_version_snapshot_onedigit() {
        let parsed = MinecraftVersion::try_from("12w03a")
            .expect("MinecraftVersion shal lbe able to parse a snapshot version string");
        assert_eq!(
            parsed,
            MinecraftVersion::Snapshot {
                year: 12,
                week: 3,
                ident: Some("a".as_bytes()[0])
            }
        )
    }

    #[test]
    fn test_version_snapshot_noident() {
        let parsed = MinecraftVersion::try_from("12w34")
            .expect("MinecraftVersion shal lbe able to parse a snapshot version string");
        assert_eq!(
            parsed,
            MinecraftVersion::Snapshot {
                year: 12,
                week: 34,
                ident: None
            }
        )
    }
}
