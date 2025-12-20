use std::path::PathBuf;

use clap::Parser;
use error::{Error, Result};

use types::{MinecraftVersion, ModDB, ModLoader, ModVersion};

mod config;
mod error;
mod manager;
mod mcmod_client;
mod solver;
mod types;

/// The options to be passed to an install or download command
#[derive(clap::Parser, Debug)]
struct Install {
    /// The config file to load. Defaults to ./mcmod.toml
    config: Option<PathBuf>,

    /// Override the default game version in the config
    #[arg(long, short = 'v', value_parser = clap::value_parser!(MinecraftVersion))]
    game_version: Option<MinecraftVersion>,

    /// Override the default mod loader in the config
    #[arg(long, short)]
    loader: Option<ModLoader>,

    /// Use the offline mod file cache
    #[arg(long)]
    offline: bool,
}

/// The options to be passed to an install or download command
#[derive(clap::Parser, Debug)]
struct Download {
    /// The config file to load. Defaults to ./mcmod.toml
    config: Option<PathBuf>,

    /// Override the default game version in the config
    #[arg(long, short = 'v', value_parser = clap::value_parser!(MinecraftVersion))]
    game_version: Option<MinecraftVersion>,

    /// Override the default mod loader in the config
    #[arg(long, short)]
    loader: Option<ModLoader>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    Download(Download),
    Install(Install),
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
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Load a config, overriding values as specified in cli
fn load_config(cli: &Cli) -> Result<config::Config> {
    let (config_path, game_version, loader) = match &cli.command {
        Command::Validate => unimplemented!("Cannot load a config for validate"),
        Command::Install(cmd) => (&cmd.config, &cmd.game_version, cmd.loader),
        Command::Download(cmd) => (&cmd.config, &cmd.game_version, cmd.loader),
    };

    let config_path = config_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("./mcmod.toml"));
    let text =
        std::fs::read_to_string(&config_path).map_err(|_| Error::ReadPath(config_path.clone()))?;
    let mut mcmod = config::Config::loads(text.as_str())?;
    game_version
        .as_ref()
        .inspect(|x| mcmod.defaults.game_version = (*x).clone());
    loader.inspect(|x| mcmod.defaults.loader = *x);
    Ok(mcmod)
}

fn solve_versions(mod_config: &config::Config) -> Result<ModDB> {
    let mut mod_solver = solver::ModSolver::new(mod_config);
    for project in mod_config.projects() {
        println!("Collecting {}", project.name);
        mod_solver
            .collect_project_and_dependencies(&project)
            .inspect(|x| println!("  Found {} projects", x.len()))
            .inspect_err(|e| println!("  Error: {e}"))?;
    }
    for project in mod_config.optional_projects() {
        println!("Collecting {} (optional)", project.name);
        let _ = mod_solver
            .collect_project_and_dependencies(&project)
            .inspect(|x| println!("  Found {} projects", x.len()))
            .inspect_err(|e| println!("  Error: {e}"));
    }
    mod_solver.solve()
}

fn solve_versions_offline(mod_config: &config::Config) -> Result<ModDB> {
    let mut mod_solver = solver::ModSolverOffline::new(mod_config)?;
    for project in mod_config.projects() {
        println!("Collecting {}", project.name);
        mod_solver
            .collect_project_and_dependencies(&project)
            .inspect(|x| println!("  Found {} projects", x.len()))
            .inspect_err(|e| println!("  Error: {e}"))?;
    }
    for project in mod_config.optional_projects() {
        println!("Collecting {} (optional)", project.name);
        let _ = mod_solver
            .collect_project_and_dependencies(&project)
            .inspect(|x| println!("  Found {} projects", x.len()))
            .inspect_err(|e| println!("  Error: {e}"));
    }
    mod_solver.solve()
}

/// Install the files from src into `dot_minecraft`, deleting any previous files in datapacks, mods,
/// and resourcepacks.
fn prepare_version_files(
    mod_manager: &manager::ModFileManager,
    mod_db: &ModDB,
    version: &ModVersion,
    install: bool,
) {
    let printed_name = mod_db
        .get_project_by_id(version.project_id)
        .map_or_else(|| version.name.as_str(), |x| x.name.as_str());
    println!(
        "Getting files for {} : {}",
        version.version_id, printed_name
    );
    for mod_file in &version.files {
        if mod_manager
            .find_file(version.version_id, &mod_file.name)
            .is_some()
        {
            println!("  Using cached file {}", mod_file.name);
        } else {
            println!("  Downloading file {}", mod_file.name);
            mod_manager
                .download_file(version.version_id, mod_file)
                .expect("Failure to download file");
        }
        if install {
            println!("  Installing");
            mod_manager
                .install_file(
                    version.version_id,
                    mod_file,
                    version.loaders.first().copied(),
                )
                .expect("Failure to get file");
        }
    }
}

fn prepare_files(mod_config: &config::Config, mod_db: &ModDB, install: bool) {
    let manager = manager::ModFileManager::new(
        mod_config.paths.data.clone(),
        mod_config.paths.dot_minecraft.clone(),
    );
    for version in mod_db.get_versions() {
        prepare_version_files(&manager, mod_db, version, install);
    }
}

fn parse_cli() -> Result<Cli> {
    let mut cli = Cli::parse();
    match &mut cli.command {
        Command::Validate => {}
        Command::Install(cmd) => {
            cmd.game_version = Some(cmd.game_version.take().unwrap().error_for_invalid()?);
        }
        Command::Download(cmd) => {
            cmd.game_version = Some(cmd.game_version.take().unwrap().error_for_invalid()?);
        }
    }
    Ok(cli)
}

fn main() {
    let cli = parse_cli().expect("Failure to parse cli");
    if cli.command.is_validate() {
        let client = mcmod_client::ModClient::new();
        let errors = client.validate_structs().expect("Failed to compare data");
        if !errors.is_empty() {
            println!("{errors:?}");
        }
    }

    let mod_config = load_config(&cli).expect("Failure to load config");
    let mod_db = if cli.command.is_offline() {
        solve_versions_offline(&mod_config).expect("Failure to resolve projects")
    } else {
        solve_versions(&mod_config).expect("Failure to resolve projects")
    };
    prepare_files(&mod_config, &mod_db, cli.command.is_install());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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

    fn load_test_config() -> config::Config {
        config::Config::loads(
            fs::read_to_string("examples/workspace_test.toml")
                .expect("Failure to read test config")
                .as_str(),
        )
        .expect("Failure to parse test config")
    }

    fn create_test_paths() {
        // Cannot remove test paths. todo: use a virtual file system so file tests can run properly
        let path = PathBuf::from(".test/.minecraft");
        fs::create_dir_all(&path)
            .unwrap_or_else(|_| panic!("Test path {} should be creatable.", path.display()));
    }

    fn check_children_count(path: &PathBuf, count: usize) {
        assert_eq!(
            path.read_dir().expect("Failure to read entries").count(),
            count,
            "Path count mismatch for {path:?}"
        );
    }

    #[test]
    fn test_action_install() {
        create_test_paths();
        let mod_config = load_test_config();
        let mod_solver = solver::ModSolver::new(&mod_config);
        let mod_db = mod_solver.solve().expect("Failure to resolve versions");
        prepare_files(&mod_config, &mod_db, true);
        let minecraft = &mod_config.paths.dot_minecraft;
        check_children_count(&minecraft.join("datapacks"), 1);
        check_children_count(&minecraft.join("mods"), 3);
        check_children_count(&minecraft.join("resourcepacks"), 1);
    }

    #[test]
    fn test_action_offline_install() {
        create_test_paths();
        let mod_config = load_test_config();
        let mod_solver = solver::ModSolver::new(&mod_config);
        let mod_db = mod_solver.solve().expect("Failure to resolve versions");
        prepare_files(&mod_config, &mod_db, false);

        let mod_solver =
            solver::ModSolverOffline::new(&mod_config).expect("Failure to load database cache");
        let mod_db = mod_solver.solve().expect("Failure to resolve versions");
        prepare_files(&mod_config, &mod_db, true);
        let minecraft = &mod_config.paths.dot_minecraft;
        check_children_count(&minecraft.join("datapacks"), 1);
        check_children_count(&minecraft.join("mods"), 3);
        check_children_count(&minecraft.join("resourcepacks"), 1);
    }
}
