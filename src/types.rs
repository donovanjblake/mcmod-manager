use crate::error::Error;
use clap;

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
