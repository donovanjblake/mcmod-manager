use crate::error::{Error, Result};
use std::hash::Hash;
use toml;

#[derive(Debug)]
pub struct Config {
    pub defaults: ConfigDefaults,
    projects: Vec<OptionConfigProject>,
    optional_projects: Vec<OptionConfigProject>,
}

impl Config {
    pub fn loads(text: String) -> Result<Config> {
        let table = text
            .parse::<toml::Table>()
            .map_err(Error::ConfigParseError)?;
        let table = MyTable { inner: &table };
        let defaults = ConfigDefaults::try_from(&table.get_table("defaults")?)?;
        let projects = table.get_table("projects")?.as_projects()?;
        let optional_projects = match table.get_table("optional-projects") {
            Ok(table) => table.as_projects()?,
            Err(Error::ConfigKeyError(_)) => Default::default(),
            Err(e) => return Err(e),
        };

        Ok(Config {
            defaults,
            projects,
            optional_projects,
        })
    }

    pub fn projects(&self) -> Vec<ConfigProject> {
        let mut result = Vec::<ConfigProject>::new();
        for project in &self.projects {
            result.push(project.resolve(&self.defaults))
        }
        result
    }
}

#[derive(Debug)]
pub struct ConfigDefaults {
    pub game_version: String,
    pub loader: String,
}

impl TryFrom<&MyTable<'_>> for ConfigDefaults {
    type Error = Error;
    fn try_from(table: &MyTable<'_>) -> std::result::Result<Self, Self::Error> {
        Ok(ConfigDefaults {
            game_version: table.get_str("game_version")?.into(),
            loader: table.get_str("loader")?.into(),
        })
    }
}

#[derive(Debug)]
pub struct ConfigProject {
    pub name: String,
    pub game_version: String,
    pub loader: String,
}

#[derive(Debug)]
struct OptionConfigProject {
    pub name: String,
    pub game_version: Option<String>,
    pub loader: Option<String>,
}

impl OptionConfigProject {
    pub fn resolve(&self, defaults: &ConfigDefaults) -> ConfigProject {
        ConfigProject {
            name: self.name.to_owned(),
            game_version: self
                .game_version
                .as_ref()
                .unwrap_or_else(|| &defaults.game_version)
                .to_owned(),
            loader: self
                .loader
                .as_ref()
                .unwrap_or_else(|| &defaults.loader)
                .to_owned(),
        }
    }
}

impl TryFrom<(&String, &MyTable<'_>)> for OptionConfigProject {
    type Error = Error;
    fn try_from(value: (&String, &MyTable<'_>)) -> std::result::Result<Self, Self::Error> {
        let name = value.0.into();
        let use_defaults = match value.1.get_bool("defaults") {
            Ok(x) => x,
            Err(Error::ConfigKeyError(_)) => false,
            Err(e) => return Err(e),
        };
        let game_version = match (value.1.get_str("game_version"), use_defaults) {
            (Ok(x), _) => Some(x.to_string()),
            (Err(Error::ConfigKeyError(_)), true) => None,
            (Err(Error::ConfigKeyError(key)), false) => {
                return Err(Error::ProjectNotResolvedError { project: name, key });
            }
            (Err(e), _) => return Err(e),
        };
        let loader = match (value.1.get_str("loader"), use_defaults) {
            (Ok(x), _) => Some(x.to_string()),
            (Err(Error::ConfigKeyError(_)), true) => None,
            (Err(Error::ConfigKeyError(key)), false) => {
                return Err(Error::ProjectNotResolvedError { project: name, key });
            }
            (Err(e), _) => return Err(e),
        };
        Ok(OptionConfigProject {
            name,
            game_version,
            loader,
        })
    }
}

struct MyTable<'t> {
    pub inner: &'t toml::Table,
}

impl<'t> From<&'t toml::Table> for MyTable<'t> {
    fn from(value: &'t toml::Table) -> Self {
        Self { inner: value }
    }
}

impl<'t> TryFrom<&'t toml::Value> for MyTable<'t> {
    type Error = Error;
    fn try_from(value: &'t toml::Value) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            inner: value.as_table().ok_or_else(|| Error::ConfigTypeError {
                key: Default::default(),
                message: "Expected a table".into(),
            })?,
        })
    }
}

impl<'t> MyTable<'t> {
    /// Get the value associated with the key
    fn get(&self, key: &String) -> Result<&'t toml::Value> {
        self.inner
            .get(key)
            .ok_or_else(|| Error::ConfigKeyError(key.into()))
    }

    /// Get the bool associated with the key
    pub fn get_bool<S>(&self, key: S) -> Result<bool>
    where
        S: ToString,
    {
        let key = key.to_string();
        self.get(&key)?
            .as_bool()
            .ok_or_else(|| Error::ConfigTypeError {
                key,
                message: "Expected a bool".into(),
            })
    }

    /// Get the str associated with the key
    pub fn get_str<S>(&self, key: S) -> Result<&'t str>
    where
        S: ToString,
    {
        let key = key.to_string();
        self.get(&key)?
            .as_str()
            .ok_or_else(|| Error::ConfigTypeError {
                key,
                message: "Expected a string".into(),
            })
    }

    /// Get the table associated with the key
    pub fn get_table<S>(&self, key: S) -> Result<Self>
    where
        S: ToString,
    {
        let key = key.to_string();
        let inner = self
            .get(&key)?
            .as_table()
            .ok_or_else(|| Error::ConfigTypeError {
                key,
                message: "Expected a table".into(),
            })?;
        Ok(Self { inner })
    }

    /// Try to get the table as a list of projects
    pub fn as_projects(&self) -> Result<Vec<OptionConfigProject>> {
        let mut result = Vec::<OptionConfigProject>::new();
        for (key, value) in self.inner.iter() {
            let table = MyTable::try_from(value).map_err(|e| match e {
                Error::ConfigTypeError { message, .. } => Error::ConfigTypeError {
                    key: key.into(),
                    message,
                },
                _ => e,
            })?;
            result.push(OptionConfigProject::try_from((key, &table))?)
        }
        Ok(result)
    }
}
