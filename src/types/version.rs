use crate::error::{Error, Result};

/// Minecraft version structure
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(try_from = "String", into = "String")]
pub enum MinecraftVersion {
    Release {
        /// Major version number
        major: u8,
        /// Minor version number
        minor: u8,
        /// Patch version number
        patch: Option<u8>,
    },
    Unknown {
        version: String,
    }
}

impl MinecraftVersion {
    /// Parse a minecraft version or return an error.
    pub fn try_parse_from(value: &String) -> Result<Self> {
        let parts: Vec<_> = value.split(&['.', '-']).collect();
        let parse_u8 = |s: &str| -> Result<u8> {
            s.parse::<u8>()
                .map_err(|_| Error::InvalidMinecraftVersion(value.clone()))
        };
        match parts.len() {
            2..4 => {
                let (major, minor) = (parse_u8(parts[0])?, parse_u8(parts[1])?);
                let patch = parts.get(2).map(|x| -> Result<Option<u8>> {
                    if x.eq_ignore_ascii_case("x") {
                        Ok(None)
                    } else {
                        Ok(Some(parse_u8(x)?))
                    }
                }).transpose()?.and_then(|x| x);
                Ok(MinecraftVersion::Release {
                    major,
                    minor,
                    patch,
                })
            }
            _ => Err(Error::InvalidMinecraftVersion(value.clone())),
        }
    }

    pub fn error_for_invalid(self) -> Result<Self> {
        match self {
            MinecraftVersion::Unknown { version } => Err(Error::InvalidMinecraftVersion(version)),
            x => Ok(x)
        }
    }
}

impl std::fmt::Display for MinecraftVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MinecraftVersion::Release { major, minor, patch } => {
                match patch {
                    Some(patch) => {
                        write!(
                            f,
                            "{major}.{minor}.{patch}",
                        )
                    }
                    None => {
                        write!(
                            f,
                            "{major}.{minor}",
                        )
        
                    }
                }
            }
            MinecraftVersion::Unknown { version } => {
                write!(f,"{version}")
            }
        }
    }
}

impl From<MinecraftVersion> for String {
    fn from(value: MinecraftVersion) -> Self {
        format!("{value}")
    }
}

impl From<&MinecraftVersion> for String {
    fn from(value: &MinecraftVersion) -> Self {
        format!("{value}")
    }
}

impl From<String> for MinecraftVersion {
    fn from(value: String) -> Self {
        MinecraftVersion::try_parse_from(&value).unwrap_or_else(|_| MinecraftVersion::Unknown { version: value })
    }
}

impl From<&str> for MinecraftVersion {
    fn from(value: &str) -> Self {
        MinecraftVersion::try_from(value.to_string()).expect("Invalid minecraft version")
    }
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
            }
        );
    }
}
