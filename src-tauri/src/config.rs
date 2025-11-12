use crate::db::{DbConnection, SettingsRepository};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Whisper,
    FunASR,
}

impl ModelType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "funasr" => ModelType::FunASR,
            _ => ModelType::Whisper,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ModelType::Whisper => "whisper".to_string(),
            ModelType::FunASR => "funasr".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub model_type: ModelType,
    pub model_name: String,
    pub is_first_launch: bool,
    pub enable_prewarming: bool,
    pub language: String,
    pub shortcut: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model_type: ModelType::Whisper,
            model_name: "small".to_string(),
            is_first_launch: true,
            enable_prewarming: true,
            language: "zh".to_string(),
            shortcut: "Cmd+Shift+S".to_string(),
        }
    }
}

pub struct ConfigManager {
    repo: SettingsRepository,
}

impl ConfigManager {
    pub fn new(conn: DbConnection) -> Self {
        Self {
            repo: SettingsRepository::new(conn),
        }
    }

    /// Load application configuration from database
    pub fn load(&self) -> Result<AppConfig, String> {
        Ok(AppConfig {
            model_type: self.get_model_type()?,
            model_name: self.get_model_name()?,
            is_first_launch: self.is_first_launch()?,
            enable_prewarming: self.is_prewarming_enabled()?,
            language: self
                .repo
                .get("language")
                .map_err(|e| e.to_string())?
                .unwrap_or_else(|| "zh".to_string()),
            shortcut: self
                .repo
                .get("shortcut")
                .map_err(|e| e.to_string())?
                .unwrap_or_else(|| "Cmd+Shift+S".to_string()),
        })
    }

    /// Get current model type (whisper or funasr)
    pub fn get_model_type(&self) -> Result<ModelType, String> {
        let value = self
            .repo
            .get("model_type")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "whisper".to_string());
        Ok(ModelType::from_str(&value))
    }

    /// Set model type
    pub fn set_model_type(&self, model_type: ModelType) -> Result<(), String> {
        self.repo
            .set("model_type", &model_type.to_string())
            .map_err(|e| e.to_string())
    }

    /// Get current model name
    pub fn get_model_name(&self) -> Result<String, String> {
        self.repo
            .get("model_name")
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Model name not set".to_string())
    }

    /// Set model name
    pub fn set_model_name(&self, name: &str) -> Result<(), String> {
        self.repo.set("model_name", name).map_err(|e| e.to_string())
    }

    /// Update model configuration
    pub fn set_model(&self, model_type: ModelType, model_name: &str) -> Result<(), String> {
        self.set_model_type(model_type)?;
        self.set_model_name(model_name)?;
        Ok(())
    }

    /// Check if this is first launch
    pub fn is_first_launch(&self) -> Result<bool, String> {
        let value = self
            .repo
            .get("is_first_launch")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "true".to_string());
        Ok(value == "true")
    }

    /// Mark first launch as complete
    pub fn mark_first_launch_complete(&self) -> Result<(), String> {
        self.repo
            .set("is_first_launch", "false")
            .map_err(|e| e.to_string())
    }

    /// Check if prewarming is enabled
    pub fn is_prewarming_enabled(&self) -> Result<bool, String> {
        let value = self
            .repo
            .get("enable_prewarming")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "true".to_string());
        Ok(value == "true")
    }

    /// Set prewarming enabled state
    pub fn set_prewarming_enabled(&self, enabled: bool) -> Result<(), String> {
        self.repo
            .set("enable_prewarming", if enabled { "true" } else { "false" })
            .map_err(|e| e.to_string())
    }

    /// Check if FunASR is being used
    pub fn is_funasr_active(&self) -> Result<bool, String> {
        Ok(self.get_model_type()? == ModelType::FunASR)
    }

    /// Check if should initialize FunASR on startup
    pub fn should_init_funasr_on_startup(&self) -> Result<bool, String> {
        Ok(self.is_funasr_active()? && !self.is_first_launch()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::path::PathBuf;

    #[test]
    fn test_config_manager() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_config.db");
        let _ = std::fs::remove_file(&db_path);

        let db = Database::new(db_path.clone()).unwrap();
        let config = ConfigManager::new(db.connection());

        // Test default values
        assert_eq!(config.get_model_type().unwrap(), ModelType::Whisper);
        assert_eq!(config.get_model_name().unwrap(), "small");
        assert!(config.is_first_launch().unwrap());

        // Test setting values
        config
            .set_model(ModelType::FunASR, "paraformer-zh")
            .unwrap();
        assert_eq!(config.get_model_type().unwrap(), ModelType::FunASR);
        assert_eq!(config.get_model_name().unwrap(), "paraformer-zh");

        // Test first launch
        config.mark_first_launch_complete().unwrap();
        assert!(!config.is_first_launch().unwrap());

        // Clean up
        let _ = std::fs::remove_file(&db_path);
    }
}
