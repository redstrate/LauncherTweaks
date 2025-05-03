use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub launcher_url: Option<String>,
    pub winhttp_proxy: Option<String>,
    #[serde(default = "Config::default_disable_webview2_install")]
    pub disable_webview2_install: bool,
    #[serde(default = "Config::default_disable_boot_version_check")]
    pub disable_boot_version_check: bool,
    #[serde(default = "Config::default_force_http")]
    pub force_http: bool,
    pub game_patch_server: Option<String>,
    pub boot_patch_server: Option<String>,
}

impl Config {
    fn default_disable_webview2_install() -> bool {
        false
    }

    fn default_disable_boot_version_check() -> bool {
        false
    }

    fn default_force_http() -> bool {
        false
    }
}

pub fn get_config() -> Config {
    std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::read_to_string(p.join("../launchertweaks.toml")).ok())
        .and_then(|s| toml::from_str::<Config>(&s).ok())
        .unwrap_or_default()
}
