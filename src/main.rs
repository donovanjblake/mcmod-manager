use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use error::{Error, Result};

mod config;
mod error;
mod labrinth;

#[derive(Parser, Debug)]
struct Cli {
    /// The config file to load. Defaults to ./mcmod.toml
    config: Option<PathBuf>,

    /// Override the default game version from the config
    #[arg(long, short = 'v')]
    game_version: Option<String>,

    /// Override the default loader from the config
    #[arg(long, short)]
    loader: Option<String>,

    /// Download the files to the given directory
    #[arg(long, short)]
    download: Option<PathBuf>,

    /// Install mods, resource packs, etc into .minecraft directory
    #[arg(long, short)]
    install: bool,
}

fn load_config(cli: &Cli) -> Result<config::Config> {
    let config_path = cli
        .config
        .to_owned()
        .unwrap_or_else(|| PathBuf::from("./mcmod.toml"));
    let mut mcmod = config::Config::loads(
        std::fs::read_to_string(config_path)
            .map_err(Error::from)?
            .as_str(),
    )?;
    cli.game_version
        .as_ref()
        .inspect(|x| mcmod.defaults.game_version = x.to_string());
    cli.loader
        .as_ref()
        .inspect(|x| mcmod.defaults.loader = x.to_string());
    Ok(mcmod)
}

fn collect_versions(
    client: &labrinth::Client,
    mcmod: &config::Config,
) -> Result<Vec<labrinth::ProjectVersion>> {
    let mut versions = Vec::<labrinth::ProjectVersion>::new();
    for project in mcmod.projects() {
        println!("Collecting {}", project.name);
        let version =
            client.get_project_version(project.name, project.game_version, project.loader)?;
        println!("  Found {}", version.name);
        versions.push(version);
    }
    Ok(versions)
}

fn collect_optional_versions(
    client: &labrinth::Client,
    mcmod: &config::Config,
) -> Vec<labrinth::ProjectVersion> {
    let mut versions = Vec::<labrinth::ProjectVersion>::new();
    for project in mcmod.optional_projects() {
        println!("Collecting {}", project.name);
        let version =
            client.get_project_version(project.name, project.game_version, project.loader);
        let version = match version {
            Ok(x) => x,
            Err(e) => {
                println!("  {e:?}");
                continue;
            }
        };
        println!("  Found {}", version.name);
        versions.push(version);
    }
    versions
}

fn init_temp(tmp: &PathBuf) -> std::io::Result<PathBuf> {
    // TODO: Make hashed temp paths to prevent collisions with other instances
    new_empty_dir(tmp)?;
    Ok(tmp.clone())
}

fn download_files(
    client: &labrinth::Client,
    versions: &Vec<labrinth::ProjectVersion>,
    path: &Path,
) -> Result<()> {
    for dir in ["mods", "resourcepacks", "datapacks"] {
        let dir = path.join(dir);
        new_empty_dir(&dir).expect("Failure to empty .minecraft sub-directory");
    }
    for version in versions {
        println!("Downloading {}", version.name);
        let files = client.download_version_files(version)?;
        let folder = match version.loaders.first().map(|x| x.as_str()) {
            Some("minecraft") => "resourcepacks",
            Some("datapack") => "datapacks",
            _ => "mods",
        };
        for (info, bytes) in files {
            let filepath = path.join(folder).join(info.filename.clone());
            fs::write(filepath, bytes).expect("Failure writing file");
        }
    }
    Ok(())
}

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

fn new_empty_dir(dir: &PathBuf) -> std::io::Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir(dir)
}

fn main() {
    let cli = Cli::parse();
    println!("{cli:?}");

    let mcmod = load_config(&cli).expect("Failure to lod config");
    println!("{mcmod:?}");

    let client = labrinth::Client::new();
    let mut versions = collect_versions(&client, &mcmod).expect("Failure to collect versions");
    versions.append(&mut collect_optional_versions(&client, &mcmod));
    let temp_path = init_temp(&mcmod.paths.temp).expect("Failure to initialize temp directory");

    if cli.install || cli.download.is_some() {
        download_files(&client, &versions, &temp_path).expect("Failure to download files");
    }

    if let Some(download_path) = cli.download {
        println!("Copying to {download_path:?}");
        new_empty_dir(&download_path).expect("Failure to remove temporary directory");
        copy_dir_all(&temp_path, &download_path)
            .expect("Failure to copy downloaded files to download directory");
    }

    if cli.install {
        println!("Installing to {:?}", mcmod.paths.dot_minecraft);
        for dir in ["mods", "resourcepacks", "datapacks"] {
            let dir = mcmod.paths.dot_minecraft.join(dir);
            new_empty_dir(&dir).expect("Failure to empty .minecraft sub-directory");
        }
        copy_dir_all(&temp_path, &mcmod.paths.dot_minecraft)
            .expect("Failure to copy install files to .minecraft directory");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_empty() {
        let cli =
            Cli::try_parse_from(["exe"]).expect("Cli shall be able to run with zero arguments");
        assert_eq!(cli.config, None);
        assert_eq!(cli.game_version, None);
        assert_eq!(cli.loader, None);
        assert_eq!(cli.download, None);
        assert_eq!(cli.install, false);
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
        assert_eq!(cli.config, Some(PathBuf::from("config")));
        assert_eq!(cli.game_version, Some(String::from("1.23.4")));
        assert_eq!(cli.loader, Some(String::from("minecraft")));
        assert_eq!(cli.download, Some(PathBuf::from("dir")));
        assert_eq!(cli.install, true);
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
        assert_eq!(cli.config, Some(PathBuf::from("config")));
        assert_eq!(cli.game_version, Some(String::from("1.23.4")));
        assert_eq!(cli.loader, Some(String::from("minecraft")));
        assert_eq!(cli.download, Some(PathBuf::from("dir")));
        assert_eq!(cli.install, true);
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
}
