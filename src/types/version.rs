use crate::error::{Error, Result};

/// Minecraft version structure
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct MinecraftVersion {
    /// Major version number
    major: u8,
    /// Minor version number
    minor: u8,
    /// Patch version number
    patch: Option<u8>,
}

impl std::fmt::Display for MinecraftVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.patch {
            Some(patch) => {
                write!(
                    f,
                    "{}.{}.{patch}",
                    self.major,
                    self.minor,
                )
            }
            None => {
                write!(
                    f,
                    "{}.{}",
                    self.major,
                    self.minor,
                )

            }
        }
    }
}

impl From<MinecraftVersion> for String {
    fn from(value: MinecraftVersion) -> Self {
        format!("{value}")
    }
}

impl TryFrom<String> for MinecraftVersion {
    type Error = Error;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
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
                Ok(MinecraftVersion {
                    major,
                    minor,
                    patch,
                })
            }
            _ => Err(Error::InvalidMinecraftVersion(value.clone())),
        }
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
            MinecraftVersion {
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
            MinecraftVersion {
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
            MinecraftVersion {
                major: 1,
                minor: 23,
                patch: None,
            }
        );
    }
}
