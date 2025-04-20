use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub launcher_url: Option<String>,
    pub winhttp_proxy: Option<String>,
}

pub fn get_config() -> Config {
    std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::read_to_string(p.join("../launchertweaks.toml")).ok())
        .and_then(|s| toml::from_str::<Config>(&s).ok())
        .unwrap_or_default()
}
