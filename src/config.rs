use std::collections::HashMap;

use crate::error::{Error, Result};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub defaults: ConfigDefaults,
    projects: HashMap<String, OptionConfigProject>,
    #[serde(default)]
    optional_projects: HashMap<String, OptionConfigProject>,
}

impl Config {
    pub fn loads(text: &str) -> Result<Config> {
        toml::from_str::<Self>(text).map_err(Error::from)
    }

    /// Gets the projects, sorted by name
    pub fn projects(&self) -> Vec<ConfigProject> {
        let mut result = Vec::<ConfigProject>::new();
        for (name, project) in &self.projects {
            result.push(project.resolve(name, &self.defaults))
        }
        result.sort_by_key(|p| p.name.clone());
        result
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConfigDefaults {
    pub game_version: String,
    pub loader: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConfigProject {
    pub name: String,
    pub game_version: String,
    pub loader: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct OptionConfigProject {
    pub game_version: Option<String>,
    pub loader: Option<String>,
}

impl OptionConfigProject {
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

    const STANDARD: &str = "\
        [defaults]
        game_version=\"1.21.2\"
        loader=\"fabric\"
        [projects]
        iris.defaults=true
        faithful-32x={defaults=true,loader=\"minecraft\"}
        ";

    const OPTIONAL: &str = "\
        [defaults]
        game_version=\"1.21.2\"
        loader=\"fabric\"
        [projects]
        iris.defaults=true
        faithful-32x={defaults=true,loader=\"minecraft\"}
        [optional-projects]
        stellaris.defaults=true
        ";

    #[test]
    fn test_parse_toml() {
        Config::loads(STANDARD).expect("Could not parse toml");
    }

    #[test]
    fn test_parse_optional() {
        Config::loads(OPTIONAL).expect("Could not parse toml");
    }

    #[test]
    fn test_set_game_version() {
        let mut config = Config::loads(STANDARD).expect("Could not parse toml");
        config.defaults.game_version = "1.21.4".into();
        let projects = config.projects();
        let expected_projects = Vec::from([
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
        assert_eq!(projects, expected_projects);
    }

    #[test]
    fn test_set_loader() {
        let mut config = Config::loads(STANDARD).expect("Could not parse toml");
        config.defaults.loader = "neoforge".into();
        let projects = config.projects();
        let expected_projects = Vec::from([
            ConfigProject {
                name: "faithful-32x".into(),
                game_version: "1.21.2".into(),
                loader: "minecraft".into(),
            },
            ConfigProject {
                name: "iris".into(),
                game_version: "1.21.2".into(),
                loader: "neoforge".into(),
            },
        ]);
        assert_eq!(projects, expected_projects);
    }

    #[test]
    fn test_get_projects() {
        let config = Config::loads(STANDARD).expect("Could not parse toml");
        let projects = config.projects();
        let expected_projects = Vec::from([
            ConfigProject {
                name: "faithful-32x".into(),
                game_version: "1.21.2".into(),
                loader: "minecraft".into(),
            },
            ConfigProject {
                name: "iris".into(),
                game_version: "1.21.2".into(),
                loader: "fabric".into(),
            },
        ]);
        assert_eq!(projects, expected_projects);
    }
}
