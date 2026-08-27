use crate::{MinecraftVersion, ModLoader};
use clap::Parser;
use std::path::PathBuf;

/// Download and install mods
#[derive(Parser, Debug)]
pub struct Install {
    /// The config file to load. Defaults to ./mcmod.toml
    pub config: Option<PathBuf>,

    /// Override the default game version in the config
    #[arg(long, short = 'v', value_parser = clap::value_parser!(MinecraftVersion))]
    pub game_version: Option<MinecraftVersion>,

    /// Override the default mod loader in the config
    #[arg(long, short)]
    pub loader: Option<ModLoader>,

    /// Use the offline mod file cache
    #[arg(long)]
    pub offline: bool,
}

/// Download mods without installing them
#[derive(Parser, Debug)]
pub struct Download {
    /// The config file to load. Defaults to ./mcmod.toml
    pub config: Option<PathBuf>,

    /// Override the default game version in the config
    #[arg(long, short = 'v', value_parser = clap::value_parser!(MinecraftVersion))]
    pub game_version: Option<MinecraftVersion>,

    /// Override the default mod loader in the config
    #[arg(long, short)]
    pub loader: Option<ModLoader>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    Download(Download),
    Install(Install),
    /// Validate program data against the database
    Validate,
}

impl Command {
    pub fn is_offline(&self) -> bool {
        match self {
            Command::Download(_) | Command::Validate => false,
            Command::Install(cmd) => cmd.offline,
        }
    }
    pub fn is_validate(&self) -> bool {
        matches!(self, Command::Validate)
    }
    pub fn is_install(&self) -> bool {
        matches!(self, Command::Install(_))
    }
}

#[derive(clap::Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_empty() {
        Cli::try_parse_from(["exe"]).expect_err("Cli shall require a command");
    }

    #[test]
    fn test_cli_parse_install_long() {
        let cli = Cli::try_parse_from([
            "exe",
            "install",
            "config",
            "--game-version",
            "1.23.4",
            "--loader",
            "minecraft",
            "--offline",
        ])
        .expect("Cli shall accept every long option");
        let Command::Install(cmd) = cli.command else {
            panic!("Cli shall parse an install command");
        };
        assert_eq!(
            cmd.config,
            Some(PathBuf::from("config")),
            "Cli shall read the input config"
        );
        assert_eq!(
            cmd.game_version,
            Some(MinecraftVersion::from("1.23.4")),
            "Cli shall read the input game version"
        );
        assert_eq!(
            cmd.loader,
            Some(ModLoader::Minecraft),
            "Cli shall read the input mod loader"
        );
        assert!(cmd.offline, "Cli shall set the offline flag");
    }

    #[test]
    fn test_cli_parse_install_short() {
        let cli = Cli::try_parse_from([
            "exe",
            "install",
            "config",
            "-v",
            "1.23.4",
            "-l",
            "minecraft",
        ])
        .expect("Cli shall accept every short option");
        let Command::Install(cmd) = cli.command else {
            panic!("Cli shall parse an install command");
        };
        assert_eq!(
            cmd.config,
            Some(PathBuf::from("config")),
            "Cli shall read the input config"
        );
        assert_eq!(
            cmd.game_version,
            Some(MinecraftVersion::from("1.23.4")),
            "Cli shall read the input game version"
        );
        assert_eq!(
            cmd.loader,
            Some(ModLoader::Minecraft),
            "Cli shall read the input mod loader"
        );
    }

    #[test]
    fn test_cli_parse_download_long() {
        let cli = Cli::try_parse_from([
            "exe",
            "download",
            "config",
            "--game-version",
            "1.23.4",
            "--loader",
            "minecraft",
        ])
        .expect("Cli shall accept every long option");
        let Command::Download(cmd) = cli.command else {
            panic!("Cli shall parse a download command");
        };
        assert_eq!(
            cmd.config,
            Some(PathBuf::from("config")),
            "Cli shall read the input config"
        );
        assert_eq!(
            cmd.game_version,
            Some(MinecraftVersion::from("1.23.4")),
            "Cli shall read the input game version"
        );
        assert_eq!(
            cmd.loader,
            Some(ModLoader::Minecraft),
            "Cli shall read the input mod loader"
        );
    }

    #[test]
    fn test_cli_parse_download_short() {
        let cli = Cli::try_parse_from([
            "exe",
            "download",
            "config",
            "-v",
            "1.23.4",
            "-l",
            "minecraft",
        ])
        .expect("Cli shall accept every short option");
        let Command::Download(cmd) = cli.command else {
            panic!("Cli shall parse a download command");
        };
        assert_eq!(
            cmd.config,
            Some(PathBuf::from("config")),
            "Cli shall read the input config"
        );
        assert_eq!(
            cmd.game_version,
            Some(MinecraftVersion::from("1.23.4")),
            "Cli shall read the input game version"
        );
        assert_eq!(
            cmd.loader,
            Some(ModLoader::Minecraft),
            "Cli shall read the input mod loader"
        );
    }
}
