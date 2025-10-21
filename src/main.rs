use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use error::Result;

use crate::types::*;

mod config;
mod error;
mod labrinth;
mod solver;
mod types;

/// The options passed to the program through the command line interface
#[derive(Parser, Debug)]
struct Cli {
    /// The config file to load. Defaults to ./mcmod.toml
    config: Option<PathBuf>,

    /// Override the default game version in the config
    #[arg(long, short = 'v', value_parser = clap::value_parser!(MinecraftVersion))]
    game_version: Option<MinecraftVersion>,

    /// Override the default mod loader in the config
    #[arg(long, short)]
    loader: Option<ModLoader>,

    /// Download the files to the given directory
    #[arg(long, short)]
    download: Option<PathBuf>,

    /// Install mods, resource packs, etc into .minecraft directory
    #[arg(long, short)]
    install: bool,

    /// Validate internal data types
    #[arg(long)]
    validate: bool,
}

/// Load a config, overriding values as specified in cli
fn load_config(cli: &Cli) -> Result<config::Config> {
    let config_path = cli
        .config
        .to_owned()
        .unwrap_or_else(|| PathBuf::from("./mcmod.toml"));
    let mut mcmod = config::Config::loads(std::fs::read_to_string(config_path)?.as_str())?;
    cli.game_version
        .inspect(|x| mcmod.defaults.game_version = *x);
    cli.loader.inspect(|x| mcmod.defaults.loader = *x);
    Ok(mcmod)
}

fn solve_versions(mod_config: &config::Config) -> Result<types::ModDB> {
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

/// Initialize the temp directory to be empty
fn init_temp(tmp: &PathBuf) -> std::io::Result<PathBuf> {
    // TODO: Make hashed temp paths to prevent collisions with other instances
    new_empty_dir(tmp)?;
    Ok(tmp.clone())
}

/// Download the files from the given versions into the given directory, deleting any previous files
/// in the directory.
fn download_files(mod_db: &ModDB, path: &Path) -> Result<()> {
    new_empty_dir(&path.to_path_buf()).expect("Failure to empty temp sub-directory");
    for dir in ["mods", "resourcepacks", "datapacks", "shaderpacks"] {
        let dir = path.join(dir);
        new_empty_dir(&dir).expect("Failure to empty temp sub-directory");
    }
    let client = labrinth::Client::new();
    for version in mod_db.get_versions() {
        let project_name = mod_db
            .get_project_by_id(&version.project_id)
            .map(|x| x.name.clone())
            .unwrap_or_else(|| "Project not cached!".into());
        println!("Downloading {} ({})", version.name, project_name);
        let files = client.download_version_files(version)?;
        let folder = match version.loaders.first() {
            Some(ModLoader::Minecraft) => "resourcepacks",
            Some(ModLoader::Datapack) => "datapacks",
            Some(ModLoader::Iris) | Some(ModLoader::Optifine) => "shaderpacks",
            _ => "mods",
        };
        for (info, bytes) in files {
            let filepath = path.join(folder).join(info.name.clone());
            fs::write(filepath, bytes).expect("Failure writing file");
        }
    }
    Ok(())
}

/// Copy all fils recursively from src to dst, creating dst and overwriting files as needed.
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    if dst.starts_with(src) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Source path should not contain file path",
        ));
    }
    if !dst.exists() {
        fs::create_dir(dst)?
    }
    for entry in src.read_dir()? {
        let entry = entry?;
        let entry_name = entry.file_name();
        let entry_name = entry_name
            .to_str()
            .ok_or_else(|| std::io::Error::other("failed to get file name"))?;
        match entry_name {
            "." => continue,
            ".." => continue,
            _ => {}
        }
        let src_path = entry.path();
        let dst_path = dst.join(entry_name);
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst.join(entry_name))?;
        } else if src_path.is_file() {
            fs::copy(&src_path, dst_path)?;
        }
    }
    Ok(())
}

/// Create dir if it does not exist, and delete any files in it.
fn new_empty_dir(dir: &PathBuf) -> std::io::Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir(dir)
}

/// Install the files from src into dot_minecraft, deleting any previous files in datapacks, mods,
/// and resourcepacks.
fn install_files(src: &PathBuf, dot_minecraft: &PathBuf) -> std::io::Result<()> {
    for dir in ["mods", "resourcepacks", "datapacks"] {
        let dir = dot_minecraft.join(dir);
        new_empty_dir(&dir)?;
    }
    copy_dir_all(src, dot_minecraft)?;
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let mod_config = load_config(&cli).expect("Failure to load config");
    if cli.validate {
        let client = labrinth::Client::new();
        let errors = client.validate_enums().expect("Failed to compare data");
        if !errors.is_empty() {
            println!("{errors:?}")
        }
    }

    let mod_db = solve_versions(&mod_config).expect("Failure to resolve projects");

    let total = mod_config.projects().len() + mod_config.optional_projects().len();
    let collected = mod_db.get_versions().len();
    println!("Found {collected}/{total} projects");

    if !cli.install && cli.download.is_none() {
        return;
    }

    let temp_path =
        init_temp(&mod_config.paths.temp).expect("Failure to initialize temp directory");
    download_files(&mod_db, &temp_path).expect("Failure to download files");

    if let Some(download_path) = cli.download.as_ref() {
        println!("Copying to {download_path:?}");
        if !download_path.try_exists().is_ok_and(|x| x) {
            fs::create_dir(download_path).expect("Failure to create download directory");
        }
        copy_dir_all(&temp_path, download_path)
            .expect("Failure to copy downloaded files to download directory");
    }

    if cli.install {
        println!("Installing to {:?}", mod_config.paths.dot_minecraft);
        install_files(&temp_path, &mod_config.paths.dot_minecraft)
            .expect("Failure to install files");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_empty() {
        let cli =
            Cli::try_parse_from(["exe"]).expect("Cli shall be able to run with zero arguments");
        assert_eq!(cli.config, None, "Cli shall set falsy defaults");
        assert_eq!(cli.game_version, None, "Cli shall set falsy defaults");
        assert_eq!(cli.loader, None, "Cli shall set falsy defaults");
        assert_eq!(cli.download, None, "Cli shall set falsy defaults");
        assert_eq!(cli.install, false, "Cli shall set falsy defaults");
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
            "dir",
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
        assert_eq!(
            cli.download,
            Some(PathBuf::from("dir")),
            "Cli shall read the input download directory"
        );
        assert_eq!(cli.install, true, "Cli shall set the install flag");
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
            "dir",
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
        assert_eq!(
            cli.download,
            Some(PathBuf::from("dir")),
            "Cli shall read the input download directory"
        );
        assert_eq!(cli.install, true, "Cli shall set the install flag");
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

    #[test]
    fn test_cli_parse_require_download_value() {
        Cli::try_parse_from(["exe", "--download"])
            .expect_err("Cli shall require a value if the --download option is specified");
    }

    #[test]
    fn test_cli_parse_require_download_value_short() {
        Cli::try_parse_from(["exe", "-d"])
            .expect_err("Cli shall require a value if the -d option is specified");
    }

    fn load_test_config() -> config::Config {
        config::Config::loads(
            fs::read_to_string("examples/integration_test.toml")
                .expect("Failure to read test config")
                .as_str(),
        )
        .expect("Failure to parse test config")
    }

    fn create_test_paths() {
        let path = PathBuf::from(".test/.minecraft");
        if !path.exists() {
            fs::create_dir_all(path).expect("Failure to create test path")
        }
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
        let mcmod = load_test_config();
        let mod_solver = solver::ModSolver::new(&mcmod);
        let moddb = mod_solver.solve().expect("Failure to resolve versions");
        let temp = init_temp(&mcmod.paths.temp).expect("Failed to initialize temp path");
        download_files(&moddb, &temp).expect("Failure to download files");
        let minecraft = &mcmod.paths.dot_minecraft;
        install_files(&temp, &minecraft).expect("Failure to install files");
        check_children_count(&minecraft.join("datapacks"), 1);
        check_children_count(&minecraft.join("mods"), 3);
        check_children_count(&minecraft.join("resourcepacks"), 1);
    }

    #[test]
    fn test_action_full_fat() {
        create_test_paths();
        let mcmod = config::Config::loads(
            fs::read_to_string("examples/mcmod.toml")
                .expect("Failure to read test config")
                .as_str(),
        )
        .expect("Failure to parse test config");
        let mod_solver = solver::ModSolver::new(&mcmod);
        let moddb = mod_solver.solve().expect("Failure to resolve versions");
        let temp = init_temp(&mcmod.paths.temp).expect("Failed to initialize temp path");
        download_files(&moddb, &temp).expect("Failure to download files");
        let minecraft = &mcmod.paths.dot_minecraft;
        install_files(&temp, &minecraft).expect("Failure to install files");
    }
}
