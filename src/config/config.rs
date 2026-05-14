use std::path::PathBuf;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub api_base_url: Option<String>,
    pub token: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let path = match Self::path() {
            Some(p) => p,
            None => return Self::default(),
        };

        if !path.exists() {
            return Self::default();
        }

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        toml::from_str(&contents).unwrap_or_default()
    }

    pub fn path() -> Option<PathBuf> {
        dirs::home_dir().map(|p| p.join(".config").join("tofu").join("config.toml"))
    }

    pub fn resolve_api_base_url(&self, cli_override: Option<String>) -> String {
        cli_override
            .or_else(|| self.api_base_url.clone())
            .unwrap_or_else(|| "https://api.trytofu.dev".to_string())
    }

    pub fn resolve_token(&self) -> Option<String> {
        self.token.clone()
    }
}
