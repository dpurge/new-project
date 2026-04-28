use std::{
    collections::HashMap,
    fs,
    io,
    path::PathBuf,
};

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub name: Option<String>,
    pub count: Option<u8>,
}

impl Config {
    pub fn load() -> Result<Self, io::Error> {
        let Some(path) = config_path() else {
            return Ok(Self::default());
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let values = parse_kv_file(&content);

        Ok(Self {
            name: values.get("name").cloned(),
            count: values.get("count").and_then(|value| value.parse::<u8>().ok()),
        })
    }
}

pub fn config_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(home.join(".config").join("{{ cookiecutter.project_slug }}").join("config.env"))
}

fn parse_kv_file(content: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };

        values.insert(key.trim().to_owned(), value.trim().to_owned());
    }

    values
}
