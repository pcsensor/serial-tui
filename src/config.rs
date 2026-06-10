use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 串口配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialSettings {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: String,
    pub stop_bits: u8,
    pub flow_control: String,
}

impl Default for SerialSettings {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: 115200,
            data_bits: 8,
            parity: "none".into(),
            stop_bits: 1,
            flow_control: "none".into(),
        }
    }
}

/// 行尾追加模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEnding {
    None,
    LF,
    CR,
    CRLF,
}

impl LineEnding {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            LineEnding::None => &[],
            LineEnding::LF => b"\n",
            LineEnding::CR => b"\r",
            LineEnding::CRLF => b"\r\n",
        }
    }
}

/// 显示格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayFormat {
    Hex,
    Ascii,
    Raw,
}

impl Default for DisplayFormat {
    fn default() -> Self {
        DisplayFormat::Ascii
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub serial: SerialSettings,
    #[serde(default = "default_line_ending")]
    pub line_ending: LineEnding,
    #[serde(default)]
    pub display_format: DisplayFormat,
}

fn default_line_ending() -> LineEnding {
    LineEnding::None
}

impl AppConfig {
    fn config_dir() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let mut path = PathBuf::from(home);
        path.push(".config");
        path.push("serial");
        path
    }

    fn config_path() -> PathBuf {
        let mut path = Self::config_dir();
        path.push("config.toml");
        path
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            let config = Self::default();
            // 首次启动时预创建默认配置目录和文件
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let content = toml::to_string_pretty(&config)?;
            std::fs::write(&path, content)?;
            return Ok(config);
        }
        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            serial: SerialSettings::default(),
            line_ending: LineEnding::None,
            display_format: DisplayFormat::Ascii,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_ending_bytes() {
        assert_eq!(LineEnding::None.as_bytes(), b"");
        assert_eq!(LineEnding::LF.as_bytes(), b"\n");
        assert_eq!(LineEnding::CR.as_bytes(), b"\r");
        assert_eq!(LineEnding::CRLF.as_bytes(), b"\r\n");
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.serial.baud_rate, 115200);
        assert_eq!(config.serial.data_bits, 8);
        assert_eq!(config.serial.parity, "none");
    }

    #[test]
    fn test_config_roundtrip() {
        let config = AppConfig {
            serial: SerialSettings {
                port: "/dev/ttyUSB0".into(),
                baud_rate: 9600,
                data_bits: 8,
                parity: "odd".into(),
                stop_bits: 2,
                flow_control: "hardware".into(),
            },
            line_ending: LineEnding::CRLF,
            display_format: DisplayFormat::Hex,
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.serial.baud_rate, 9600);
        assert_eq!(parsed.line_ending, LineEnding::CRLF);
        assert_eq!(parsed.display_format, DisplayFormat::Hex);
    }
}
