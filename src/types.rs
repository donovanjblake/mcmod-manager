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
pub struct MinecraftVersion {
    /// Major version number
    major: u8,
    /// Minor version number
    minor: u8,
    /// Patch version number
    patch: Option<u8>,
    /// Release candidate number
    rc: Option<u8>,
}

impl std::fmt::Display for MinecraftVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}{}",
            self.major,
            self.minor,
            self.patch
                .map_or_else(|| String::from("x"), |x| x.to_string()),
            self.rc.map_or(String::new(), |x| format!("-rc{x}"))
        )
    }
}

impl From<MinecraftVersion> for String {
    fn from(value: MinecraftVersion) -> Self {
        format!(
            "{}.{}.{}{}",
            value.major,
            value.minor,
            value
                .patch
                .map_or_else(|| String::from("x"), |x| x.to_string()),
            value.rc.map_or(String::new(), |x| format!("-rc{x}"))
        )
    }
}

impl TryFrom<String> for MinecraftVersion {
    type Error = Error;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let parts: Vec<_> = value.split(".").collect();
        if parts.len() < 2 || 3 < parts.len() {
            return Err(Error::InvalidMinecraftVersion(value.to_string()));
        }
        let parse_u8 = |s: &str| -> Result<u8> {
            s.parse::<u8>()
                .map_err(|_| Error::InvalidMinecraftVersion(value.to_string()))
        };
        let (major, minor) = (parse_u8(parts[0])?, parse_u8(parts[1])?);
        let (patch, rc) = if parts.get(2).map_or("x", |x| *x).eq_ignore_ascii_case("x") {
            (None, None)
        } else {
            let parts: Vec<_> = parts.get(2).unwrap().split("-rc").collect();
            let patch = Some(parse_u8(parts[0])?);
            let rc = if let Some(s) = parts.get(1) {
                Some(parse_u8(s)?)
            } else {
                None
            };
            (patch, rc)
        };
        Ok(MinecraftVersion {
            major,
            minor,
            patch,
            rc,
        })
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

#[cfg(test)]
mod tests {
    use crate::types::MinecraftVersion;

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
                rc: None,
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
            MinecraftVersion {
                major: 1,
                minor: 23,
                patch: Some(4),
                rc: Some(5),
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

                rc: None,
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
                rc: None,
            }
        );
    }
}
