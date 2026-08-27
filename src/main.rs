use std::path::PathBuf;

use clap::Parser;
use error::{Error, Result};

use command_line::{Cli, Command};
use types::{MinecraftVersion, ModDB, ModLoader, ModVersion};

mod command_line;
mod config;
mod error;
mod manager;
mod mcmod_client;
mod solver;
mod types;

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

/// Get all the appropriate mods for the given config
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

/// Get all the appropriate mods for the given config using the offline cache
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
            if let Some(game_version) = cmd.game_version.take() {
                cmd.game_version = Some(game_version.error_for_invalid()?);
            }
        }
        Command::Download(cmd) => {
            if let Some(game_version) = cmd.game_version.take() {
                cmd.game_version = Some(game_version.error_for_invalid()?);
            }
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
