use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_launcher_url")]
    pub launcher_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            launcher_url: Self::default_launcher_url(),
        }
    }
}

impl Config {
    fn default_launcher_url() -> String {
        "about:blank".to_string()
    }
}

pub fn get_config() -> Config {
    std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::read_to_string(p.join("../launchertweaks.toml")).ok())
        .and_then(|s| toml::from_str::<Config>(&s).ok())
        .unwrap_or_default()
}
