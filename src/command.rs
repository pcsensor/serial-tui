use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 快捷指令条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub name: String,
    pub data: String,
}

/// 快捷指令列表
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandList {
    pub commands: Vec<CommandEntry>,
}

impl CommandList {
    fn storage_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("serial");
        path.push("commands.json");
        path
    }

    pub fn load() -> Result<Self> {
        let path = Self::storage_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let list: CommandList = serde_json::from_str(&content)?;
        Ok(list)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::storage_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn add(&mut self, name: String, data: String) {
        self.commands.push(CommandEntry { name, data });
    }

    pub fn remove(&mut self, index: usize) -> Option<CommandEntry> {
        if index < self.commands.len() {
            Some(self.commands.remove(index))
        } else {
            None
        }
    }

    pub fn update(&mut self, index: usize, name: String, data: String) -> bool {
        if index < self.commands.len() {
            self.commands[index] = CommandEntry { name, data };
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let mut list = CommandList::default();
        list.add("AT".into(), "AT\r\n".into());
        assert_eq!(list.len(), 1);
        assert_eq!(list.commands[0].name, "AT");
        assert_eq!(list.commands[0].data, "AT\r\n");
    }

    #[test]
    fn test_remove() {
        let mut list = CommandList::default();
        list.add("CMD1".into(), "data1".into());
        list.add("CMD2".into(), "data2".into());
        let removed = list.remove(0).unwrap();
        assert_eq!(removed.name, "CMD1");
        assert_eq!(list.len(), 1);
        assert_eq!(list.commands[0].name, "CMD2");
    }

    #[test]
    fn test_remove_invalid_index() {
        let mut list = CommandList::default();
        assert!(list.remove(0).is_none());
    }

    #[test]
    fn test_update() {
        let mut list = CommandList::default();
        list.add("CMD1".into(), "old".into());
        assert!(list.update(0, "CMD1_NEW".into(), "new".into()));
        assert_eq!(list.commands[0].name, "CMD1_NEW");
        assert_eq!(list.commands[0].data, "new");
        assert!(!list.update(99, "x".into(), "y".into()));
    }

    #[test]
    fn test_json_roundtrip() {
        let mut list = CommandList::default();
        list.add("AT".into(), "AT\r\n".into());
        list.add("AT+RST".into(), "AT+RST\r\n".into());
        let json = serde_json::to_string(&list).unwrap();
        let parsed: CommandList = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.commands[0].name, "AT");
    }
}
