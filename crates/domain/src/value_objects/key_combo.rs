use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Modifiers {
    pub logo: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        logo: false,
        alt: false,
        ctrl: false,
        shift: false,
    };

    pub const fn new(logo: bool, alt: bool, ctrl: bool, shift: bool) -> Self {
        Self {
            logo,
            alt,
            ctrl,
            shift,
        }
    }

    pub fn is_empty(self) -> bool {
        !self.logo && !self.alt && !self.ctrl && !self.shift
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    pub modifiers: Modifiers,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyComboParseError {
    EmptyString,
    MissingKey,
}

impl std::fmt::Display for KeyComboParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyString => write!(f, "Key combo string is empty"),
            Self::MissingKey => write!(f, "Key combo is missing the base key"),
        }
    }
}

impl std::error::Error for KeyComboParseError {}

impl KeyCombo {
    pub fn new(modifiers: Modifiers, key: impl Into<String>) -> Self {
        Self {
            modifiers,
            key: key.into().to_lowercase(),
        }
    }

    pub fn matches(&self, modifiers: Modifiers, key: &str) -> bool {
        self.modifiers == modifiers && self.key.eq_ignore_ascii_case(key)
    }

    /// Parses key combos like "Super + Return", "Super+Shift+Q", "Ctrl+Alt+t", "Super+1".
    pub fn parse(s: &str) -> Result<Self, KeyComboParseError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(KeyComboParseError::EmptyString);
        }

        let parts: Vec<&str> = trimmed
            .split('+')
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();

        if parts.is_empty() {
            return Err(KeyComboParseError::EmptyString);
        }

        let mut modifiers = Modifiers::NONE;
        let mut key: Option<String> = None;

        for part in parts {
            let lower = part.to_ascii_lowercase();
            match lower.as_str() {
                "super" | "logo" | "mod4" | "win" => modifiers.logo = true,
                "alt" | "mod1" => modifiers.alt = true,
                "ctrl" | "control" => modifiers.ctrl = true,
                "shift" => modifiers.shift = true,
                _ => {
                    key = Some(part.to_string());
                }
            }
        }

        let key = key.ok_or(KeyComboParseError::MissingKey)?;
        Ok(Self::new(modifiers, key))
    }
}

impl std::fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if self.modifiers.logo {
            parts.push("Super");
        }
        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        parts.push(&self.key);
        write!(f, "{}", parts.join("+"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_combo() {
        let combo = KeyCombo::parse("Super + Return").unwrap();
        assert!(combo.modifiers.logo);
        assert!(!combo.modifiers.shift);
        assert_eq!(combo.key, "return");
        assert!(combo.matches(Modifiers::new(true, false, false, false), "RETURN"));

        let combo2 = KeyCombo::parse("Super+Shift+Q").unwrap();
        assert!(combo2.modifiers.logo);
        assert!(combo2.modifiers.shift);
        assert_eq!(combo2.key, "q");

        let combo3 = KeyCombo::parse("Ctrl + Alt + Delete").unwrap();
        assert!(combo3.modifiers.ctrl);
        assert!(combo3.modifiers.alt);
        assert_eq!(combo3.key, "delete");

        assert!(KeyCombo::parse("").is_err());
        assert!(KeyCombo::parse("Super+Shift+").is_err());
    }

    #[test]
    fn test_key_combo_display() {
        let combo = KeyCombo::parse("Super+Shift+Q").unwrap();
        assert_eq!(combo.to_string(), "Super+Shift+q");
    }
}
