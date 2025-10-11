use std::{collections::HashMap, path::PathBuf};

use crate::error::{Error, Result};

/// Configuration containing paths and projects to use
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    /// Default values to use for projects
    pub defaults: ConfigDefaults,

    /// Paths to use
    #[serde(default)]
    pub paths: ConfigPaths,

    /// Projects that must be available
    projects: HashMap<String, OptionConfigProject>,

    /// Projects that may be available
    #[serde(default, rename = "optional-projects")]
    optional_projects: HashMap<String, OptionConfigProject>,
}

impl Config {
    /// Load the config from TOML text
    pub fn loads(text: &str) -> Result<Config> {
        let result = toml::from_str::<Self>(text).map_err(Error::from)?;
        if !result.paths.dot_minecraft.is_dir() {
            panic!("{:?}: directory does not exist", result.paths.dot_minecraft);
        }
        Ok(result)
    }

    /// Get the projects, sorted by name
    pub fn projects(&self) -> Vec<ConfigProject> {
        let mut result = Vec::<ConfigProject>::new();
        for (name, project) in &self.projects {
            result.push(project.resolve(name, &self.defaults))
        }
        result.sort_by_key(|p| p.name.clone());
        result
    }

    /// Get the optional projects, sorted by name
    pub fn optional_projects(&self) -> Vec<ConfigProject> {
        let mut result = Vec::<ConfigProject>::new();
        for (name, project) in &self.optional_projects {
            result.push(project.resolve(name, &self.defaults))
        }
        result.sort_by_key(|p| p.name.clone());
        result
    }
}

/// Get the data directory for this program's data
fn default_data() -> PathBuf {
    dirs::data_local_dir()
        .expect("Could not locate local data, please specify paths.data in config")
        .join("mcmod")
}

/// Get the .minecraft directory
fn default_dot_minecraft() -> PathBuf {
    if cfg!(windows) {
        dirs::data_dir()
    } else {
        dirs::home_dir()
    }
    .and_then(|x| {
        let x = x.join(".minecraft");
        if x.exists() { Some(x) } else { None }
    })
    .expect("Could not locate .minecraft, plese specify paths.dot_minecraft in config")
}

/// Get the temp directory for this program's data
fn default_temp() -> PathBuf {
    std::env::temp_dir().join("mcmod")
}

/// Default targets for projects
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConfigDefaults {
    /// Target Minecraft version
    pub game_version: String,

    /// Mod loader
    pub loader: String,
}

/// Paths to use
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct ConfigPaths {
    /// This program's data directory
    #[serde(default = "default_data")]
    pub data: PathBuf,

    /// .minecraft directory
    #[serde(default = "default_dot_minecraft")]
    pub dot_minecraft: PathBuf,

    /// This program's temp directry
    #[serde(default = "default_temp")]
    pub temp: PathBuf,
}

impl Default for ConfigPaths {
    fn default() -> Self {
        Self {
            dot_minecraft: default_dot_minecraft(),
            temp: default_temp(),
            data: default_data(),
        }
    }
}

/// Project information given to the caller, fully populated
#[derive(Debug, PartialEq, Eq)]
pub struct ConfigProject {
    /// Name (or id) of the project
    pub name: String,

    /// Target Minecraft version
    pub game_version: String,

    /// Target mod loader
    pub loader: String,
}

/// Internal project information. Use [OptionConfigProject::resolve] to replace `None` at runtime.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct OptionConfigProject {
    /// Target Minecraft version
    pub game_version: Option<String>,

    /// Target mod loader
    pub loader: Option<String>,
}

impl OptionConfigProject {
    /// Return a project populated with defaults instead of Nones
    pub fn resolve(&self, name: &String, defaults: &ConfigDefaults) -> ConfigProject {
        ConfigProject {
            name: name.to_owned(),
            game_version: self
                .game_version
                .as_ref()
                .unwrap_or(&defaults.game_version)
                .to_owned(),
            loader: self.loader.as_ref().unwrap_or(&defaults.loader).to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_test_config() -> Config {
        Config::loads(
            std::fs::read_to_string("examples/integration_test.toml")
                .expect("Failure to read test config")
                .as_str(),
        )
        .expect("Config shall be able to parse a toml.")
    }

    fn create_test_paths() {
        let path = PathBuf::from(".test/.minecraft");
        if !path.exists() {
            std::fs::create_dir_all(path).expect("Failure to create test path")
        }
    }

    #[test]
    fn test_set_game_version() {
        create_test_paths();
        let mut config = load_test_config();
        config.defaults.game_version = "1.21.4".into();
        let projects = config.projects();
        let expected_projects = Vec::from([
            ConfigProject {
                name: "blazeandcaves-advancements-pack".into(),
                game_version: "1.21.4".into(),
                loader: "datapack".into(),
            },
            ConfigProject {
                name: "faithful-32x".into(),
                game_version: "1.21.4".into(),
                loader: "minecraft".into(),
            },
            ConfigProject {
                name: "iris".into(),
                game_version: "1.21.4".into(),
                loader: "fabric".into(),
            },
        ]);
        assert_eq!(
            projects, expected_projects,
            "Config shall return projects with the new default game version."
        );
    }

    #[test]
    fn test_set_loader() {
        create_test_paths();
        let mut config = load_test_config();
        config.defaults.loader = "neoforge".into();
        let projects = config.projects();
        let expected_projects = Vec::from([
            ConfigProject {
                name: "blazeandcaves-advancements-pack".into(),
                game_version: "1.21.5".into(),
                loader: "datapack".into(),
            },
            ConfigProject {
                name: "faithful-32x".into(),
                game_version: "1.21.5".into(),
                loader: "minecraft".into(),
            },
            ConfigProject {
                name: "iris".into(),
                game_version: "1.21.5".into(),
                loader: "neoforge".into(),
            },
        ]);
        assert_eq!(
            projects, expected_projects,
            "Config shall return projects with the new default mod loader."
        );
    }

    #[test]
    fn test_get_projects() {
        create_test_paths();
        let config = load_test_config();
        let projects = config.projects();
        let expected_projects = Vec::from([
            ConfigProject {
                name: "blazeandcaves-advancements-pack".into(),
                game_version: "1.21.5".into(),
                loader: "datapack".into(),
            },
            ConfigProject {
                name: "faithful-32x".into(),
                game_version: "1.21.5".into(),
                loader: "minecraft".into(),
            },
            ConfigProject {
                name: "iris".into(),
                game_version: "1.21.5".into(),
                loader: "fabric".into(),
            },
        ]);
        assert_eq!(
            projects, expected_projects,
            "Config shall return projects with the default game version and mod loader."
        );
    }

    #[test]
    fn test_get_optional_projects() {
        create_test_paths();
        let config = load_test_config();
        let projects = config.optional_projects();
        let expected_projects = Vec::from([
            ConfigProject {
                name: "camps_castles_carriages".into(),
                game_version: "1.21.5".into(),
                loader: "fabric".into(),
            },
            ConfigProject {
                name: "lithium".into(),
                game_version: "1.21.5".into(),
                loader: "fabric".into(),
            },
        ]);
        assert_eq!(
            projects, expected_projects,
            "Config shall return projects with the default game version and mod loader."
        );
    }
}
