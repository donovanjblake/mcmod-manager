use std::{fs, path::PathBuf};

use clap::Parser;

mod config;
mod error;
mod labrinth;

#[derive(Parser, Debug)]
struct Cli {
    /// The config file to load. Defaults to ./mcmod.toml
    config: Option<PathBuf>,

    /// Override the default game version from the config
    #[arg(long)]
    game_version: Option<String>,

    /// Override the default loader from the config
    #[arg(long)]
    loader: Option<String>,

    /// Download the files to the given directory
    #[arg(long, short = 'd')]
    download: Option<PathBuf>,

    /// Respond 'yes' to removing files in the given directory
    #[arg(long, short = 'y')]
    yes: bool,
}

fn main() {
    let cli = Cli::parse();
    println!("{cli:?}");

    let config_path = cli.config.unwrap_or_else(|| PathBuf::from("./mcmod.toml"));
    let mut mod_config = config::Config::loads(
        std::fs::read_to_string(config_path)
            .expect("Failure reading config file")
            .as_str(),
    )
    .expect("Failure parsing config file");
    cli.game_version
        .inspect(|x| mod_config.defaults.game_version = x.into());
    cli.loader
        .inspect(|x| mod_config.defaults.loader = x.into());
    println!("{mod_config:?}");

    let client = labrinth::Client::new();
    let mut versions = Vec::<labrinth::ProjectVersion>::new();
    for project in mod_config.projects() {
        println!("Collecting {}", project.name);
        let version = client
            .get_project_version(project.name, project.game_version, project.loader)
            .expect("Failure collecting project");
        println!("  Found {}", version.name);
        versions.push(version);
    }

    if let Some(download_path) = cli.download {
        if download_path.is_dir() && !cli.yes {
            panic!(
                "Download directory already exists. Pass --yes or -y if you are sure you want to replace it. All existing files will be removed!"
            )
        } else if download_path.exists() {
            fs::remove_dir_all(&download_path).expect("Failed to empty download path");
        }
        fs::create_dir(&download_path).expect("Failed to create download directory");
        for version in versions {
            println!("Downloading {}", version.name);
            let files = client
                .download_version_files(&version)
                .expect("Failure downloading file");
            for (info, bytes) in files {
                let filepath = download_path.join(info.filename.clone());
                fs::write(filepath, bytes).expect("Failure writing file");
            }
        }
    }
}
