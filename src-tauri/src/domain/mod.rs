pub mod catalog;
pub mod recipes;
pub mod reliability;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message_key: String,
    pub recoverable: bool,
}
impl AppError {
    pub fn new(code: &str, message_key: &str, recoverable: bool) -> Self {
        Self {
            code: code.into(),
            message_key: message_key.into(),
            recoverable,
        }
    }
    pub fn storage() -> Self {
        Self::new("STORAGE_UNAVAILABLE", "errors.storage", true)
    }
    pub fn integrity() -> Self {
        Self::new("SCHEMA_INTEGRITY", "errors.integrity", false)
    }
}
impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::SqliteFailure(failure, _) => match failure.code {
                rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase => {
                    Self::integrity()
                }
                rusqlite::ErrorCode::ConstraintViolation => {
                    Self::new("INVALID_INPUT", "errors.validation", true)
                }
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked => {
                    Self::new("STORAGE_BUSY", "errors.busy", true)
                }
                _ => Self::storage(),
            },
            _ => Self::storage(),
        }
    }
}
impl From<std::io::Error> for AppError {
    fn from(_: std::io::Error) -> Self {
        Self::storage()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}
impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "system" => Ok(Self::System),
            _ => Err(AppError::integrity()),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub theme: Theme,
    pub locale: String,
}
impl Preferences {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.locale != "tr" {
            return Err(AppError::new("INVALID_INPUT", "errors.validation", true));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageStatus {
    pub schema_version: i64,
    pub sqlite_version: String,
    pub foreign_keys: bool,
    pub fts5: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub preferences: Preferences,
    pub storage: StorageStatus,
}
