// SPDX-License-Identifier: Apache-2.0

//! Dotted IDs for rules and fix actions, such as `storage.root_almost_full`.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Category;

/// Why a text is not a valid dotted ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdError {
    /// The ID is not in the form `<category>.<name>`.
    BadShape(String),
    /// The part before the dot is not a known category.
    UnknownCategory(String),
    /// The name must start with `a-z` and use only `a-z`, `0-9`, and `_`.
    BadName(String),
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdError::BadShape(id) => write!(f, "'{id}' is not in the form <category>.<name>"),
            IdError::UnknownCategory(c) => write!(f, "'{c}' is not a known category"),
            IdError::BadName(n) => {
                write!(f, "'{n}' must start with a-z and use only a-z, 0-9 and _")
            }
        }
    }
}

impl std::error::Error for IdError {}

/// Checks `<category>.<name>` and returns the category.
fn parse(id: &str) -> Result<Category, IdError> {
    let (category, name) = id
        .split_once('.')
        .ok_or_else(|| IdError::BadShape(id.to_owned()))?;
    if name.contains('.') {
        return Err(IdError::BadShape(id.to_owned()));
    }
    let category = category
        .parse()
        .map_err(|_| IdError::UnknownCategory(category.to_owned()))?;
    let starts_with_letter = name.chars().next().is_some_and(|c| c.is_ascii_lowercase());
    let allowed = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !starts_with_letter || !allowed {
        return Err(IdError::BadName(name.to_owned()));
    }
    Ok(category)
}

macro_rules! dotted_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name {
            id: String,
            category: Category,
        }

        impl $name {
            /// Checks the text and wraps it.
            pub fn new(id: impl Into<String>) -> Result<Self, IdError> {
                let id = id.into();
                let category = parse(&id)?;
                Ok(Self { id, category })
            }

            /// The ID as text.
            pub fn as_str(&self) -> &str {
                &self.id
            }

            /// The category named before the dot.
            pub fn category(&self) -> Category {
                self.category
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdError;

            fn try_from(id: String) -> Result<Self, Self::Error> {
                Self::new(id)
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.id)
            }
        }
    };
}

dotted_id! {
    /// The ID of a rule, such as `storage.root_almost_full`.
    RuleId
}

dotted_id! {
    /// The ID of a fix action, such as `storage.clean_apt_cache`.
    FixActionId
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_id_keeps_its_category() {
        let id = RuleId::new("storage.root_almost_full").unwrap();
        assert_eq!(id.category(), Category::Storage);
        assert_eq!(id.as_str(), "storage.root_almost_full");
        assert_eq!(id.to_string(), "storage.root_almost_full");
    }

    #[test]
    fn bad_ids_are_rejected() {
        let bad_shape = ["storage", "storage.a.b", ".name", "storage."];
        for id in bad_shape {
            assert!(RuleId::new(id).is_err(), "{id} should fail");
        }
        assert_eq!(
            RuleId::new("disk.full"),
            Err(IdError::UnknownCategory("disk".into()))
        );
        for name in ["Full", "1st", "root-full", "root full"] {
            assert_eq!(
                FixActionId::new(format!("storage.{name}")),
                Err(IdError::BadName(name.into()))
            );
        }
    }

    #[test]
    fn serde_uses_plain_text_and_validates() {
        let id = FixActionId::new("storage.clean_apt_cache").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"storage.clean_apt_cache\"");
        assert_eq!(serde_json::from_str::<FixActionId>(&json).unwrap(), id);
        assert!(serde_json::from_str::<FixActionId>("\"nope\"").is_err());
    }
}
