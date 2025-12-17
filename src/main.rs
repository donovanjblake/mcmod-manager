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

/// The options passed to the program through the command line interface
#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// The config file to load. Defaults to ./mcmod.toml
    config: Option<PathBuf>,

    /// Override the default game version in the config
    #[arg(long, short = 'v', value_parser = clap::value_parser!(MinecraftVersion))]
    game_version: Option<MinecraftVersion>,

    /// Override the default mod loader in the config
    #[arg(long, short)]
    loader: Option<ModLoader>,

    /// Download the mod fles without installing them
    #[arg(long, short)]
    download: bool,

    /// Install mods, resource packs, etc into .minecraft directory
    #[arg(long, short)]
    install: bool,

    /// Use the offline mod file cache
    #[arg(long)]
    offline: bool,

    /// Validate internal data types
    #[arg(long)]
    validate: bool,
}

/// Load a config, overriding values as specified in cli
fn load_config(cli: &Cli) -> Result<config::Config> {
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| PathBuf::from("./mcmod.toml"));
    let text = std::fs::read_to_string(&config_path).map_err(|_| Error::ReadPath(config_path.clone()))?;
    let mut mcmod = config::Config::loads(text.as_str())?;
    cli.game_version
        .as_ref()
        .inspect(|x| mcmod.defaults.game_version = (*x).clone());
    cli.loader.inspect(|x| mcmod.defaults.loader = *x);
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
    if cli.game_version.is_some() {
        cli.game_version = Some(cli.game_version.take().unwrap().error_for_invalid()?);
    }
    Ok(cli)
}

fn main() {
    let cli = parse_cli().expect("Failure to parse cli");
    let mod_config = load_config(&cli).expect("Failure to load config");
    if cli.validate {
        let client = mcmod_client::ModClient::new();
        let errors = client.validate_structs().expect("Failed to compare data");
        if !errors.is_empty() {
            println!("{errors:?}");
        }
    }

    let mod_db = if cli.offline {
        solve_versions_offline(&mod_config).expect("Failure to resolve projects")
    } else {
        solve_versions(&mod_config).expect("Failure to resolve projects")
    };
    if cli.download || cli.install {
        prepare_files(&mod_config, &mod_db, cli.install);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_cli_parse_empty() {
        let cli =
            Cli::try_parse_from(["exe"]).expect("Cli shall be able to run with zero arguments");
        assert_eq!(cli.config, None, "Cli shall set falsy defaults");
        assert_eq!(cli.game_version, None, "Cli shall set falsy defaults");
        assert_eq!(cli.loader, None, "Cli shall set falsy defaults");
        assert!(!cli.download, "Cli shall set falsy defaults");
        assert!(!cli.install, "Cli shall set falsy defaults");
    }

    #[test]
    fn test_cli_parse_all() {
        let cli = Cli::try_parse_from([
            "exe",
            "config",
            "--game-version",
            "1.23.4",
            "--loader",
            "minecraft",
            "--download",
            "--install",
        ])
        .expect("Cli shall accept every long option");
        assert_eq!(
            cli.config,
            Some(PathBuf::from("config")),
            "Cli shall read the input config"
        );
        assert_eq!(
            cli.game_version,
            Some(MinecraftVersion::from("1.23.4")),
            "Cli shall read the input game version"
        );
        assert_eq!(
            cli.loader,
            Some(ModLoader::Minecraft),
            "Cli shall read the input mod loader"
        );
        assert!(cli.download, "Cli shall set the download flag");
        assert!(cli.install, "Cli shall set the install flag");
    }

    #[test]
    fn test_cli_parse_short() {
        let cli = Cli::try_parse_from([
            "exe",
            "config",
            "-v",
            "1.23.4",
            "-l",
            "minecraft",
            "-d",
            "-i",
        ])
        .expect("Cli shall accept every short option");
        assert_eq!(
            cli.config,
            Some(PathBuf::from("config")),
            "Cli shall read the input config"
        );
        assert_eq!(
            cli.game_version,
            Some(MinecraftVersion::from("1.23.4")),
            "Cli shall read the input game version"
        );
        assert_eq!(
            cli.loader,
            Some(ModLoader::Minecraft),
            "Cli shall read the input mod loader"
        );
        assert!(cli.download, "Cli shall set the install flag");
        assert!(cli.install, "Cli shall set the install flag");
    }

    #[test]
    fn test_cli_parse_require_game_value_version() {
        Cli::try_parse_from(["exe", "--game-version"])
            .expect_err("Cli shall require a value if the --game-version option is specified");
    }

    #[test]
    fn test_cli_parse_require_game_value_version_short() {
        Cli::try_parse_from(["exe", "-v"])
            .expect_err("Cli shall require a value if the -v option is specified");
    }

    #[test]
    fn test_cli_parse_require_loader_value() {
        Cli::try_parse_from(["exe", "--loader"])
            .expect_err("Cli shall require a value if the --loader option is specified");
    }

    #[test]
    fn test_cli_parse_require_loader_value_short() {
        Cli::try_parse_from(["exe", "-l"])
            .expect_err("Cli shall require a value if the -l option is specified");
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
