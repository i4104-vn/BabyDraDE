//! Autostart configuration defining shell commands executed when BabyDraDE starts.

use serde::{Deserialize, Serialize};

fn default_autostart_commands() -> Vec<String> {
    vec![
        "babydra-desktop".to_string(),
        "babydra-topbar".to_string(),
    ]
}

/// Autostart configuration model matching `[autostart]` in `~/.babydra/babydra.conf`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AutostartConfig {
    /// List of shell commands to execute upon desktop environment startup.
    #[serde(default = "default_autostart_commands")]
    pub commands: Vec<String>,
}

impl Default for AutostartConfig {
    fn default() -> Self {
        Self {
            commands: default_autostart_commands(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autostart_config_defaults() {
        let config = AutostartConfig::default();
        assert_eq!(
            config.commands,
            vec!["babydra-desktop".to_string(), "babydra-topbar".to_string()]
        );
    }

    #[test]
    fn test_autostart_config_toml_roundtrip() {
        let toml_str = r#"
            commands = ["babydra-desktop", "babydra-topbar", "my-app --daemon"]
        "#;
        let config: AutostartConfig = toml::from_str(toml_str).expect("deserialize autostart");
        assert_eq!(config.commands.len(), 3);
        assert_eq!(config.commands[2], "my-app --daemon");

        let serialized = toml::to_string(&config).expect("serialize autostart");
        let deserialized: AutostartConfig =
            toml::from_str(&serialized).expect("deserialize roundtrip");
        assert_eq!(config, deserialized);
    }
}
