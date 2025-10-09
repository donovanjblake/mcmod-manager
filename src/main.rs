use std::path::PathBuf;

use clap::Parser;

mod labrinth;
mod config;
mod error;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,

    #[arg(long)]
    game_version: Option<String>,

    #[arg(long)]
    loader: Option<String>,

    #[arg(long, short = 'd')]
    download: bool,
}

fn main() {
    let cli = Cli::parse();
    // println!("{cli:?}");
    let config_path = cli.config.unwrap_or_else(|| PathBuf::from("./mcmod.toml"));
    let mut mod_config = config::Config::loads(
        std::fs::read_to_string(config_path)
            .expect("Failed to read file")
            .as_str(),
    )
    .expect("Failed to parse config");
    cli.game_version
        .inspect(|x| mod_config.defaults.game_version = x.into());
    cli.loader
        .inspect(|x| mod_config.defaults.loader = x.into());
    println!("{mod_config:?}");
    let client = labrinth::Client::new();
    for project in mod_config.projects() {
        let version = client
            .get_project_version(project.name, project.game_version, project.loader)
            .expect("Failed to get project");
        println!("{version:?}");
    }
}
