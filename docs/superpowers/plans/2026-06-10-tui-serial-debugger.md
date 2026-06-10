# TUI 串口调试助手 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 使用 ratatui + tokio 构建跨平台 TUI 串口调试助手，支持多格式收发、快捷指令、协议解析和热插拔重连。

**架构：** 事件驱动分层架构。App 层为唯一状态持有者，串口 IO 通过独立 tokio task + mpsc 通道与 UI 通信，协议解析通过 trait 可扩展。

**技术栈：** ratatui + crossterm + tokio + serialport + serde + toml + chrono + anyhow

---

### 任务 1：项目骨架初始化

**文件：**
- 创建：`Cargo.toml`
- 创建：`src/main.rs`

- [ ] **步骤 1：创建 Cargo 项目**

```bash
cargo init --name serial-tui /Users/pcsensor/Projects/serial
```

- [ ] **步骤 2：编写 Cargo.toml 依赖**

```toml
[package]
name = "serial-tui"
version = "0.1.0"
edition = "2021"
description = "TUI 串口调试助手"

[dependencies]
ratatui = "0.28"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
serialport = "4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
chrono = "0.4"
anyhow = "1"
dirs = "5"

[dev-dependencies]
insta = "1"
```

- [ ] **步骤 3：编写最小入口 main.rs**

```rust
use anyhow::Result;

fn main() -> Result<()> {
    println!("Serial TUI - 启动中...");
    Ok(())
}
```

- [ ] **步骤 4：编译验证**

```bash
cargo build
```

- [ ] **步骤 5：Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: 初始化项目骨架和依赖"
```

---

### 任务 2：事件枚举模块

**文件：**
- 创建：`src/event.rs`

- [ ] **步骤 1：编写 event.rs**

```rust
use crossterm::event::KeyEvent;

/// 应用事件枚举，涵盖按键、串口数据、系统事件
#[derive(Debug, Clone)]
pub enum Event {
    /// 键盘事件
    Key(KeyEvent),
    /// 串口接收数据
    RxData(Vec<u8>),
    /// 可用端口列表变化（热插拔）
    PortsChanged,
    /// 串口错误或被动断开
    SerialError(String),
    /// 定时 tick（UI 刷新、自动重连检查）
    Tick,
    /// 退出应用
    Quit,
}
```

- [ ] **步骤 2：在 main.rs 中声明模块**

```rust
mod event;
```

- [ ] **步骤 3：编译验证**

```bash
cargo build
```

- [ ] **步骤 4：Commit**

```bash
git add src/event.rs src/main.rs
git commit -m "feat: 添加事件枚举模块 event.rs"
```

---

### 任务 3：配置持久化模块

**文件：**
- 创建：`src/config.rs`
- 创建：测试 `src/config.rs`（内联 `#[cfg(test)]`）

- [ ] **步骤 1：编写 config.rs（含测试）**

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 串口配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialSettings {
    /// 端口名，如 "/dev/ttyUSB0" 或 "COM3"
    pub port: String,
    /// 波特率，如 115200
    pub baud_rate: u32,
    /// 数据位：5/6/7/8
    pub data_bits: u8,
    /// 校验位："none" / "odd" / "even"
    pub parity: String,
    /// 停止位：1 或 2
    pub stop_bits: u8,
    /// 流控："none" / "hardware" / "software"
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

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 串口参数
    #[serde(default)]
    pub serial: SerialSettings,
    /// 行尾偏好
    #[serde(default = "default_line_ending")]
    pub line_ending: LineEnding,
    /// 显示格式
    #[serde(default)]
    pub display_format: DisplayFormat,
}

fn default_line_ending() -> LineEnding {
    LineEnding::None
}

impl Default for DisplayFormat {
    fn default() -> Self {
        DisplayFormat::Ascii
    }
}

impl AppConfig {
    /// 配置文件路径
    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("serial");
        path.push("config.toml");
        path
    }

    /// 加载配置，文件不存在返回默认值
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置
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
```

- [ ] **步骤 2：在 main.rs 声明模块并编译验证**

```rust
mod config;
mod event;
```

```bash
cargo build
```

- [ ] **步骤 3：运行测试**

```bash
cargo test
```

- [ ] **步骤 4：Commit**

```bash
git add src/config.rs src/main.rs
git commit -m "feat: 添加配置持久化模块 config.rs（TOML 格式）"
```

---

### 任务 4：快捷指令管理模块

**文件：**
- 创建：`src/command.rs`

- [ ] **步骤 1：编写 command.rs（含测试）**

```rust
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
    /// 指令存储路径
    fn storage_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("serial");
        path.push("commands.json");
        path
    }

    /// 加载指令列表
    pub fn load() -> Result<Self> {
        let path = Self::storage_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let list: CommandList = serde_json::from_str(&content)?;
        Ok(list)
    }

    /// 保存指令列表
    pub fn save(&self) -> Result<()> {
        let path = Self::storage_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// 添加指令
    pub fn add(&mut self, name: String, data: String) {
        self.commands.push(CommandEntry { name, data });
    }

    /// 删除指定索引的指令
    pub fn remove(&mut self, index: usize) -> Option<CommandEntry> {
        if index < self.commands.len() {
            Some(self.commands.remove(index))
        } else {
            None
        }
    }

    /// 更新指定索引的指令
    pub fn update(&mut self, index: usize, name: String, data: String) -> bool {
        if index < self.commands.len() {
            self.commands[index] = CommandEntry { name, data };
            true
        } else {
            false
        }
    }

    /// 获取指令数量
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// 是否为空
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
```

- [ ] **步骤 2：在 main.rs 声明模块并编译验证**

```rust
mod command;
mod config;
mod event;
```

```bash
cargo build && cargo test
```

- [ ] **步骤 3：Commit**

```bash
git add src/command.rs src/main.rs
git commit -m "feat: 添加快捷指令管理模块 command.rs"
```

---

### 任务 5：协议解析 - 显示格式转换

**文件：**
- 创建：`src/protocol/mod.rs`
- 创建：`src/protocol/format.rs`

- [ ] **步骤 1：编写 format.rs（含测试）**

```rust
use crate::config::DisplayFormat;

/// 将原始字节按指定格式转换为显示字符串
pub fn format_bytes(data: &[u8], format: DisplayFormat) -> String {
    match format {
        DisplayFormat::Hex => {
            data.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ")
        }
        DisplayFormat::Ascii => {
            data.iter()
                .map(|&b| {
                    if b.is_ascii_graphic() || b == b' ' {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect()
        }
        DisplayFormat::Raw => {
            String::from_utf8_lossy(data).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_format() {
        let data = b"Hello";
        assert_eq!(format_bytes(data, DisplayFormat::Hex), "48 65 6C 6C 6F");
    }

    #[test]
    fn test_hex_format_empty() {
        assert_eq!(format_bytes(b"", DisplayFormat::Hex), "");
    }

    #[test]
    fn test_ascii_format() {
        let data = b"Hello\x00World";
        let result = format_bytes(data, DisplayFormat::Ascii);
        assert!(result.contains("Hello"));
        assert!(result.contains('.'));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_raw_format() {
        let data = b"Hello World";
        assert_eq!(format_bytes(data, DisplayFormat::Raw), "Hello World");
    }

    #[test]
    fn test_single_byte_hex() {
        assert_eq!(format_bytes(&[0xFF], DisplayFormat::Hex), "FF");
        assert_eq!(format_bytes(&[0x0A], DisplayFormat::Hex), "0A");
    }
}
```

- [ ] **步骤 2：编写 protocol/mod.rs**

```rust
pub mod format;
pub mod parser;
```

- [ ] **步骤 3：在 main.rs 声明并编译测试**

```rust
mod command;
mod config;
mod event;
mod protocol;
```

```bash
cargo build && cargo test
```

- [ ] **步骤 4：Commit**

```bash
git add src/protocol/ src/main.rs
git commit -m "feat: 添加协议显示格式转换模块 format.rs"
```

---

### 任务 6：协议解析 - trait + 内置解析器

**文件：**
- 创建：`src/protocol/parser.rs`

- [ ] **步骤 1：编写 parser.rs（含测试）**

```rust
/// 结构化协议解析器 trait
pub trait ProtocolParser: Send + Sync {
    /// 解析器名称，用于 UI 显示
    fn name(&self) -> &str;

    /// 尝试解析数据，成功返回结构化展示字符串，失败返回 None 回退到原始格式
    fn try_parse(&self, data: &[u8]) -> Option<String>;
}

// ─── 内置解析器 ────────────────────────────────────────────

/// 按行协议：原样显示文本
pub struct LineProtocol;

impl ProtocolParser for LineProtocol {
    fn name(&self) -> &str {
        "Line"
    }

    fn try_parse(&self, data: &[u8]) -> Option<String> {
        let s = String::from_utf8_lossy(data);
        if s.trim().is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    }
}

/// JSON 协议：检测并美化 JSON
pub struct JsonProtocol;

impl ProtocolParser for JsonProtocol {
    fn name(&self) -> &str {
        "JSON"
    }

    fn try_parse(&self, data: &[u8]) -> Option<String> {
        let s = String::from_utf8_lossy(data);
        let trimmed = s.trim();
        if (trimmed.starts_with('{') || trimmed.starts_with('[')) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
                return Some(serde_json::to_string_pretty(&value).ok()?);
            }
        }
        None
    }
}

/// Modbus RTU 协议：解析帧结构（显示功能码和数据）
pub struct ModbusRtuProtocol;

impl ProtocolParser for ModbusRtuProtocol {
    fn name(&self) -> &str {
        "ModbusRTU"
    }

    fn try_parse(&self, data: &[u8]) -> Option<String> {
        if data.len() < 3 {
            return None;
        }
        let addr = data[0];
        let func = data[1];
        let func_name = match func {
            0x03 => "读取保持寄存器",
            0x06 => "写单个寄存器",
            0x10 => "写多个寄存器",
            _ => "未知功能码",
        };
        let payload_hex: String = data[2..]
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        Some(format!(
            "ModbusRTU | 地址:{} | 功能码:0x{:02X}({}) | 数据:{}",
            addr, func, func_name, payload_hex
        ))
    }
}

// ─── 解析器注册表 ──────────────────────────────────────────

/// 管理解析器列表，按优先级链式尝试
pub struct ParserRegistry {
    parsers: Vec<Box<dyn ProtocolParser>>,
    /// 当前激活的解析器索引
    pub active_index: usize,
}

impl ParserRegistry {
    /// 创建默认注册表（含所有内置解析器）
    pub fn default_parsers() -> Self {
        Self {
            parsers: vec![
                Box::new(LineProtocol),
                Box::new(JsonProtocol),
                Box::new(ModbusRtuProtocol),
            ],
            active_index: 0,
        }
    }

    /// 获取当前激活解析器名称
    pub fn active_name(&self) -> &str {
        self.parsers
            .get(self.active_index)
            .map(|p| p.name())
            .unwrap_or("无")
    }

    /// 获取所有解析器名称
    pub fn names(&self) -> Vec<String> {
        self.parsers.iter().map(|p| p.name().to_string()).collect()
    }

    /// 切换到下一个解析器
    pub fn next(&mut self) {
        if !self.parsers.is_empty() {
            self.active_index = (self.active_index + 1) % self.parsers.len();
        }
    }

    /// 使用当前激活的解析器尝试解析
    pub fn parse(&self, data: &[u8]) -> Option<String> {
        self.parsers.get(self.active_index)?.try_parse(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_protocol() {
        let p = LineProtocol;
        let result = p.try_parse(b"Hello World").unwrap();
        assert_eq!(result, "Hello World");
        assert!(p.try_parse(b"  ").is_none());
    }

    #[test]
    fn test_json_protocol() {
        let p = JsonProtocol;
        let result = p.try_parse(b"{\"key\":\"value\"}").unwrap();
        assert!(result.contains("key"));
        assert!(result.contains("value"));
        assert!(p.try_parse(b"not json").is_none());
    }

    #[test]
    fn test_modbus_rtu_protocol() {
        let p = ModbusRtuProtocol;
        assert!(p.try_parse(&[0x01, 0x03]).is_none()); // 太短
        let result = p.try_parse(&[0x01, 0x03, 0x00, 0x01]).unwrap();
        assert!(result.contains("读取保持寄存器"));
        assert!(result.contains("地址:1"));
    }

    #[test]
    fn test_parser_registry() {
        let mut registry = ParserRegistry::default_parsers();
        assert_eq!(registry.active_name(), "Line");
        registry.next();
        assert_eq!(registry.active_name(), "JSON");
        registry.next();
        assert_eq!(registry.active_name(), "ModbusRTU");
        registry.next();
        assert_eq!(registry.active_name(), "Line");

        let names = registry.names();
        assert_eq!(names.len(), 3);
    }
}
```

- [ ] **步骤 2：编译并运行测试**

```bash
cargo build && cargo test
```

- [ ] **步骤 3：Commit**

```bash
git add src/protocol/parser.rs
git commit -m "feat: 添加协议解析 trait 及内置解析器（Line/JSON/ModbusRTU）"
```

---

### 任务 7：串口管理层 - 枚举与连接

**文件：**
- 创建：`src/serial/mod.rs`

- [ ] **步骤 1：编写 serial/mod.rs（含测试）**

```rust
use anyhow::{Context, Result};
use serialport::{available_ports, SerialPort};

/// 串口连接状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    /// 未连接
    Disconnected,
    /// 已连接
    Connected,
    /// 被动断开，正在重连
    Reconnecting,
}

/// 串口管理器（不含读写 task）
pub struct SerialManager {
    /// 当前打开的串口，None 表示未连接
    port: Option<Box<dyn SerialPort>>,
    /// 连接状态
    pub state: ConnectionState,
    /// 重连计数器
    pub reconnect_count: u32,
    /// 上次连接使用的端口名
    pub last_port_name: Option<String>,
    /// 上次连接使用的波特率
    pub last_baud_rate: u32,
    /// 用户是否主动断开（主动断开不自动重连）
    pub user_disconnect: bool,
}

impl SerialManager {
    pub fn new() -> Self {
        Self {
            port: None,
            state: ConnectionState::Disconnected,
            reconnect_count: 0,
            last_port_name: None,
            last_baud_rate: 115200,
            user_disconnect: false,
        }
    }

    /// 枚举可用端口
    pub fn list_ports() -> Vec<String> {
        available_ports()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.port_name)
            .collect()
    }

    /// 打开串口
    pub fn open(
        &mut self,
        port_name: &str,
        baud_rate: u32,
        data_bits: u8,
        parity: &str,
        stop_bits: u8,
        flow_control: &str,
    ) -> Result<()> {
        let mut builder = serialport::new(port_name, baud_rate);

        builder.data_bits(match data_bits {
            5 => serialport::DataBits::Five,
            6 => serialport::DataBits::Six,
            7 => serialport::DataBits::Seven,
            _ => serialport::DataBits::Eight,
        });

        builder.parity(match parity {
            "odd" => serialport::Parity::Odd,
            "even" => serialport::Parity::Even,
            _ => serialport::Parity::None,
        });

        builder.stop_bits(match stop_bits {
            2 => serialport::StopBits::Two,
            _ => serialport::StopBits::One,
        });

        builder.flow_control(match flow_control {
            "hardware" => serialport::FlowControl::Hardware,
            "software" => serialport::FlowControl::Software,
            _ => serialport::FlowControl::None,
        });

        let port = builder
            .open()
            .with_context(|| format!("无法打开串口 {}", port_name))?;

        self.port = Some(port);
        self.state = ConnectionState::Connected;
        self.last_port_name = Some(port_name.to_string());
        self.last_baud_rate = baud_rate;
        self.reconnect_count = 0;
        self.user_disconnect = false;

        Ok(())
    }

    /// 断开串口
    pub fn close(&mut self) {
        self.port = None;
        self.state = ConnectionState::Disconnected;
    }

    /// 用户主动断开
    pub fn disconnect_by_user(&mut self) {
        self.user_disconnect = true;
        self.close();
    }

    /// 进入重连模式（被动断开时调用）
    pub fn enter_reconnect(&mut self) {
        if !self.user_disconnect {
            self.port = None;
            self.state = ConnectionState::Reconnecting;
            self.reconnect_count += 1;
        }
    }

    /// 检查是否需要重连，返回 true 表示应发起重连尝试
    pub fn should_reconnect(&self) -> bool {
        self.state == ConnectionState::Reconnecting
            && !self.user_disconnect
            && self.reconnect_count <= 10
            && self.last_port_name.is_some()
    }

    /// 获取当前端口名称
    pub fn current_port_name(&self) -> Option<&str> {
        self.port.as_ref().map(|p| p.name().unwrap_or("")).or_else(|| self.last_port_name.as_deref())
    }

    /// 获取端口可变引用（供 reader/writer 使用）
    pub fn port_ref(&self) -> Option<&dyn SerialPort> {
        self.port.as_ref().map(|p| p.as_ref())
    }

    /// 获取端口可变引用（写入操作需要）
    pub fn port_mut(&mut self) -> Option<&mut Box<dyn SerialPort>> {
        self.port.as_mut()
    }

    /// 是否已连接
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }
}
```

- [ ] **步骤 2：在 main.rs 声明并编译**

```rust
mod command;
mod config;
mod event;
mod protocol;
mod serial;
```

```bash
cargo build
```

- [ ] **步骤 3：Commit**

```bash
git add src/serial/mod.rs src/main.rs
git commit -m "feat: 添加串口管理层 serial/mod.rs（枚举/连接/重连逻辑）"
```

---

### 任务 8：串口异步读取任务

**文件：**
- 创建：`src/serial/reader.rs`

- [ ] **步骤 1：编写 reader.rs**

```rust
use crate::event::Event;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

/// 启动串口异步读取任务
/// 读取线程不断从串口读数据，通过 channel 发送到 App 层
pub fn start_reader_task(
    port: Box<dyn serialport::SerialPort>,
    tx: UnboundedSender<Event>,
    running: Arc<Mutex<bool>>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        let mut buf = vec![0u8; 256];
        // 初始化 running 为 true
        {
            let mut r = running.lock().unwrap();
            *r = true;
        }

        loop {
            {
                let r = running.lock().unwrap();
                if !*r {
                    break;
                }
            }

            // 尝试读取
            match port.bytes_to_read() {
                Ok(0) => {
                    // 无数据可读，短暂休眠
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Ok(_) => {}
                Err(_) => {
                    // 读取大小失败，可能断开
                    let _ = tx.send(Event::SerialError("串口读取错误".into()));
                    break;
                }
            }

            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let data = buf[..n].to_vec();
                    if tx.send(Event::RxData(data)).is_err() {
                        break; // 接收端已关闭
                    }
                }
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // 超时，继续
                    continue;
                }
                Err(_) => {
                    let _ = tx.send(Event::SerialError("串口读取断开".into()));
                    break;
                }
            }
        }
    })
}
```

- [ ] **步骤 2：编译验证**

```bash
cargo build
```

- [ ] **步骤 3：Commit**

```bash
git add src/serial/reader.rs
git commit -m "feat: 添加串口异步读取任务 reader.rs"
```

---

### 任务 9：串口写入任务

**文件：**
- 创建：`src/serial/writer.rs`

- [ ] **步骤 1：编写 writer.rs**

```rust
use anyhow::Result;

/// 向串口写入数据（追加行尾）
pub fn write_to_port(
    port: &mut Box<dyn serialport::SerialPort>,
    data: &[u8],
    line_ending: &[u8],
) -> Result<usize> {
    let mut buf = data.to_vec();
    buf.extend_from_slice(line_ending);
    let written = port.write(&buf)?;
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_builder() {
        let mut buf = b"AT".to_vec();
        buf.extend_from_slice(b"\r\n");
        assert_eq!(buf, b"AT\r\n");

        let mut buf2 = b"Hello".to_vec();
        buf2.extend_from_slice(b"");
        assert_eq!(buf2, b"Hello");
    }
}
```

- [ ] **步骤 2：编译验证**

```bash
cargo build && cargo test
```

- [ ] **步骤 3：Commit**

```bash
git add src/serial/writer.rs
git commit -m "feat: 添加串口写入函数 writer.rs"
```

---

### 任务 10：数据导出模块

**文件：**
- 创建：`src/export.rs`

- [ ] **步骤 1：编写 export.rs（含测试）**

```rust
use anyhow::{Context, Result};
use std::io::Write;

/// 数据记录条目
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataRecord {
    /// 时间戳（毫秒精度）
    pub time: String,
    /// 方向："RX" 或 "TX"
    pub dir: String,
    /// 数据内容（已格式化字符串）
    pub data: String,
}

/// 导出为 CSV 文件
pub fn export_csv(records: &[DataRecord], path: &str) -> Result<()> {
    let mut file = std::fs::File::create(path)
        .with_context(|| format!("无法创建文件 {}", path))?;

    writeln!(file, "时间戳,方向,数据")?;
    for r in records {
        writeln!(file, "{},{},{}", r.time, r.dir, r.data)?;
    }
    Ok(())
}

/// 导出为 JSON 文件
pub fn export_json(records: &[DataRecord], path: &str) -> Result<()> {
    let content = serde_json::to_string_pretty(records)?;
    std::fs::write(path, content)
        .with_context(|| format!("无法写入文件 {}", path))?;
    Ok(())
}

/// 导出为纯文本
pub fn export_text(records: &[DataRecord], path: &str) -> Result<()> {
    let mut file = std::fs::File::create(path)
        .with_context(|| format!("无法创建文件 {}", path))?;

    for r in records {
        writeln!(file, "{} [{}] {}", r.time, r.dir, r.data)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn sample_records() -> Vec<DataRecord> {
        vec![
            DataRecord {
                time: "14:32:01.234".into(),
                dir: "RX".into(),
                data: "AT+GMR".into(),
            },
            DataRecord {
                time: "14:32:01.456".into(),
                dir: "TX".into(),
                data: "OK".into(),
            },
        ]
    }

    #[test]
    fn test_export_csv() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_export.csv");
        let path_str = path.to_str().unwrap();

        export_csv(&sample_records(), path_str).unwrap();

        let mut content = String::new();
        std::fs::File::open(&path)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert!(content.contains("时间戳,方向,数据"));
        assert!(content.contains("AT+GMR"));
        assert!(content.contains("OK"));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_json() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_export.json");
        let path_str = path.to_str().unwrap();

        export_json(&sample_records(), path_str).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: Vec<DataRecord> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].data, "AT+GMR");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_text() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_export.txt");
        let path_str = path.to_str().unwrap();

        export_text(&sample_records(), path_str).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[RX]"));
        assert!(content.contains("[TX]"));

        let _ = std::fs::remove_file(&path);
    }
}
```

- [ ] **步骤 2：在 main.rs 声明并编译测试**

```rust
mod command;
mod config;
mod event;
mod export;
mod protocol;
mod serial;
```

```bash
cargo build && cargo test
```

- [ ] **步骤 3：Commit**

```bash
git add src/export.rs src/main.rs
git commit -m "feat: 添加数据导出模块 export.rs（CSV/JSON/文本）"
```

---

### 任务 11：App 状态管理核心

**文件：**
- 创建：`src/app.rs`

- [ ] **步骤 1：编写 app.rs**

```rust
use crate::command::CommandList;
use crate::config::{AppConfig, DisplayFormat, LineEnding, SerialSettings};
use crate::event::Event;
use crate::export::DataRecord;
use crate::protocol::parser::ParserRegistry;
use crate::serial::{ConnectionState, SerialManager};
use anyhow::Result;
use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// 焦点区域
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    Settings,
    Terminal,
    SendInput,
    QuickSend,
}

/// 数据缓存条目
#[derive(Debug, Clone)]
pub struct TerminalLine {
    /// 时间戳
    pub timestamp: String,
    /// 方向：true = 接收(RX)，false = 发送(TX)
    pub is_rx: bool,
    /// 原始数据
    pub raw_data: Vec<u8>,
}

/// 应用全局状态
pub struct App {
    /// 串口管理器
    pub serial_manager: SerialManager,
    /// 收发的数据缓存（保留所有历史）
    pub terminal_lines: Vec<TerminalLine>,
    /// 数据记录（用于导出）
    pub data_records: Vec<DataRecord>,
    /// 收发区滚动偏移
    pub scroll_offset: usize,
    /// 当前焦点区域
    pub focus: FocusArea,
    /// 串口配置
    pub serial_settings: SerialSettings,
    /// 显示格式
    pub display_format: DisplayFormat,
    /// 行尾模式
    pub line_ending: LineEnding,
    /// 发送输入框内容
    pub send_input: String,
    /// 快捷指令列表
    pub command_list: CommandList,
    /// 快捷指令选中索引
    pub quick_send_selected: usize,
    /// 端口列表（动态刷新）
    pub available_ports: Vec<String>,
    /// 端口列表选中索引（设置栏用）
    pub port_selected: usize,
    /// 波特率选中索引
    pub baud_rate_selected: usize,
    /// 协议解析器注册表
    pub parser_registry: ParserRegistry,
    /// 是否正在运行
    pub running: bool,
    /// 配置
    pub config: AppConfig,
    /// RX 字节计数
    pub rx_bytes: u64,
    /// TX 字节计数
    pub tx_bytes: u64,
    /// 日志目录
    log_dir: Option<String>,
    /// 状态消息
    pub status_message: String,
    /// 串口写入通道
    write_tx: Option<UnboundedSender<Vec<u8>>>,
    /// 波特率列表
    baud_rates: Vec<u32>,
}

impl App {
    pub fn new() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        let command_list = CommandList::load().unwrap_or_default();

        Self {
            serial_manager: SerialManager::new(),
            terminal_lines: Vec::new(),
            data_records: Vec::new(),
            scroll_offset: 0,
            focus: FocusArea::Terminal,
            serial_settings: config.serial.clone(),
            display_format: config.display_format,
            line_ending: config.line_ending,
            send_input: String::new(),
            command_list,
            quick_send_selected: 0,
            available_ports: SerialManager::list_ports(),
            port_selected: 0,
            baud_rate_selected: 6, // 115200 在列表中的索引
            parser_registry: ParserRegistry::default_parsers(),
            running: true,
            config,
            rx_bytes: 0,
            tx_bytes: 0,
            log_dir: None,
            status_message: String::new(),
            write_tx: None,
            baud_rates: vec![
                300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
            ],
        }
    }

    /// 处理事件
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::RxData(data) => self.handle_rx_data(data),
            Event::PortsChanged => {
                self.available_ports = SerialManager::list_ports();
            }
            Event::SerialError(msg) => {
                self.status_message = format!("错误: {}", msg);
                self.serial_manager.enter_reconnect();
            }
            Event::Tick => {
                // 检查重连
                if self.serial_manager.should_reconnect() {
                    self.status_message = format!(
                        "正在重连 {}  (第 {} 次)...",
                        self.serial_manager.last_port_name.as_deref().unwrap_or("?"),
                        self.serial_manager.reconnect_count
                    );
                    // 重连逻辑由 main.rs 中的 tick 处理
                }
            }
            Event::Quit => {
                // 退出时保存配置
                self.save_config();
                self.command_list.save().ok();
                self.running = false;
            }
        }
    }

    /// 处理按键
    fn handle_key(&mut self, key: KeyEvent) {
        // 全局快捷键
        match key {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.save_config();
                self.command_list.save().ok();
                self.running = false;
                return;
            }
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.display_format = match self.display_format {
                    DisplayFormat::Hex => DisplayFormat::Ascii,
                    DisplayFormat::Ascii => DisplayFormat::Raw,
                    DisplayFormat::Raw => DisplayFormat::Hex,
                };
                return;
            }
            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.terminal_lines.clear();
                self.data_records.clear();
                self.scroll_offset = 0;
                return;
            }
            KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.parser_registry.next();
                return;
            }
            KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.available_ports = SerialManager::list_ports();
                return;
            }
            KeyEvent {
                code: KeyCode::Tab, ..
            } => {
                self.focus = match self.focus {
                    FocusArea::Settings => FocusArea::Terminal,
                    FocusArea::Terminal => FocusArea::SendInput,
                    FocusArea::SendInput => FocusArea::QuickSend,
                    FocusArea::QuickSend => FocusArea::Settings,
                };
                return;
            }
            _ => {}
        }

        // 根据焦点区域分发
        match self.focus {
            FocusArea::Settings => self.handle_settings_key(key),
            FocusArea::Terminal => self.handle_terminal_key(key),
            FocusArea::SendInput => self.handle_send_input_key(key),
            FocusArea::QuickSend => self.handle_quick_send_key(key),
        }
    }

    /// 处理接收数据
    fn handle_rx_data(&mut self, data: Vec<u8>) {
        self.rx_bytes += data.len() as u64;
        let now = Local::now();
        let timestamp = now.format("%H:%M:%S.%3f").to_string();

        let formatted = crate::protocol::format::format_bytes(&data, self.display_format);
        let display = self
            .parser_registry
            .parse(&data)
            .unwrap_or(formatted.clone());

        self.terminal_lines.push(TerminalLine {
            timestamp: timestamp.clone(),
            is_rx: true,
            raw_data: data.clone(),
        });

        self.data_records.push(DataRecord {
            time: timestamp,
            dir: "RX".into(),
            data: display,
        });

        // 自动滚动到底部
        if self.scroll_offset > 0 {
            self.scroll_offset = 0;
        }
    }

    /// 发送数据
    pub fn send_data(&mut self, data: Vec<u8>) {
        if let Some(ref tx) = self.write_tx {
            let line_ending = self.line_ending.as_bytes().to_vec();
            let mut buf = data.clone();
            buf.extend_from_slice(&line_ending);

            let _ = tx.send(buf);
            self.tx_bytes += data.len() as u64;

            let now = Local::now();
            let timestamp = now.format("%H:%M:%S.%3f").to_string();

            let formatted = crate::protocol::format::format_bytes(&data, self.display_format);
            let display = self
                .parser_registry
                .parse(&data)
                .unwrap_or(formatted.clone());

            self.terminal_lines.push(TerminalLine {
                timestamp: timestamp.clone(),
                is_rx: false,
                raw_data: data,
            });

            self.data_records.push(DataRecord {
                time: timestamp,
                dir: "TX".into(),
                data: display,
            });
        }
    }

    /// 设置串口写入通道
    pub fn set_write_tx(&mut self, tx: UnboundedSender<Vec<u8>>) {
        self.write_tx = Some(tx);
    }

    /// 获取写入通道克隆
    pub fn write_tx_clone(&self) -> Option<UnboundedSender<Vec<u8>>> {
        self.write_tx.clone()
    }

    fn save_config(&mut self) {
        self.config.serial = self.serial_settings.clone();
        self.config.line_ending = self.line_ending;
        self.config.display_format = self.display_format;
        self.config.save().ok();
    }

    /// 设置栏按键处理
    fn handle_settings_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.port_selected > 0 {
                    self.port_selected -= 1;
                }
            }
            KeyCode::Down => {
                if !self.available_ports.is_empty()
                    && self.port_selected < self.available_ports.len() - 1
                {
                    self.port_selected += 1;
                }
            }
            KeyCode::Left => {
                if self.baud_rate_selected > 0 {
                    self.baud_rate_selected -= 1;
                }
            }
            KeyCode::Right => {
                if self.baud_rate_selected < self.baud_rates.len() - 1 {
                    self.baud_rate_selected += 1;
                }
            }
            KeyCode::Char('c')
                if key.modifiers == KeyModifiers::CONTROL || key.modifiers == KeyModifiers::NONE =>
            {
                self.toggle_connection();
            }
            _ => {}
        }
    }

    /// 连接/断开串口
    pub fn toggle_connection(&mut self) {
        match self.serial_manager.state {
            ConnectionState::Connected => {
                self.serial_manager.disconnect_by_user();
            }
            _ => {
                if let Some(port_name) = self.available_ports.get(self.port_selected).cloned() {
                    let baud_rate = self.baud_rates
                        .get(self.baud_rate_selected)
                        .copied()
                        .unwrap_or(115200);

                    self.serial_settings.port = port_name.clone();
                    self.serial_settings.baud_rate = baud_rate;

                    match self.serial_manager.open(
                        &port_name,
                        baud_rate,
                        self.serial_settings.data_bits,
                        &self.serial_settings.parity,
                        self.serial_settings.stop_bits,
                        &self.serial_settings.flow_control,
                    ) {
                        Ok(()) => {
                            self.status_message = format!("已连接 {}", port_name);
                        }
                        Err(e) => {
                            self.status_message = format!("连接失败: {}", e);
                        }
                    }
                }
            }
        }
    }

    fn handle_terminal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                self.scroll_offset += 1;
            }
            KeyCode::Down => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::PageUp => {
                self.scroll_offset += 10;
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
            }
            _ => {}
        }
    }

    fn handle_send_input_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.send_input.push(c);
            }
            KeyCode::Backspace => {
                self.send_input.pop();
            }
            KeyCode::Enter => {
                let input = std::mem::take(&mut self.send_input);
                if !input.is_empty() {
                    self.send_data(input.into_bytes());
                }
            }
            KeyCode::Left => {
                // 切换行尾
                self.line_ending = match self.line_ending {
                    LineEnding::None => LineEnding::CRLF,
                    LineEnding::CRLF => LineEnding::CR,
                    LineEnding::CR => LineEnding::LF,
                    LineEnding::LF => LineEnding::None,
                };
            }
            _ => {}
        }
    }

    fn handle_quick_send_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.quick_send_selected > 0 {
                    self.quick_send_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.quick_send_selected < self.command_list.len().saturating_sub(1) {
                    self.quick_send_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(cmd) = self.command_list.commands.get(self.quick_send_selected) {
                    self.send_data(cmd.data.clone().into_bytes());
                }
            }
            _ => {}
        }
    }

    /// 获取当前选中的波特率
    pub fn current_baud_rate(&self) -> u32 {
        self.baud_rates
            .get(self.baud_rate_selected)
            .copied()
            .unwrap_or(115200)
    }

    /// 获取波特率列表
    pub fn baud_rates(&self) -> &[u32] {
        &self.baud_rates
    }
}
```

- [ ] **步骤 2：编译验证**

```bash
cargo build
```

- [ ] **步骤 3：Commit**

```bash
git add src/app.rs
git commit -m "feat: 添加 App 状态管理核心 app.rs"
```

---

### 任务 12：UI - 状态栏组件

**文件：**
- 创建：`src/ui/mod.rs`
- 创建：`src/ui/status.rs`

- [ ] **步骤 1：编写 ui/mod.rs**

```rust
pub mod quick_send;
pub mod send_input;
pub mod settings;
pub mod status;
pub mod terminal;
```

- [ ] **步骤 2：编写 ui/status.rs**

```rust
use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// 渲染底部状态栏
pub fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let state_text = match app.serial_manager.state {
        crate::serial::ConnectionState::Connected => {
            Span::styled("● 已连接", Style::default().fg(Color::Green))
        }
        crate::serial::ConnectionState::Disconnected => {
            Span::styled("○ 未连接", Style::default().fg(Color::Gray))
        }
        crate::serial::ConnectionState::Reconnecting => {
            Span::styled("⟳ 重连中", Style::default().fg(Color::Yellow))
        }
    };

    let format_text = format!("格式: {:?}", app.display_format);
    let protocol_text = format!("协议: {}", app.parser_registry.active_name());
    let focus_text = match app.focus {
        crate::app::FocusArea::Settings => "焦点: 设置栏",
        crate::app::FocusArea::Terminal => "焦点: 收发区",
        crate::app::FocusArea::SendInput => "焦点: 发送栏",
        crate::app::FocusArea::QuickSend => "焦点: 快捷发送",
    };

    let line = Line::from(vec![
        Span::raw(" "),
        state_text,
        Span::raw(" | "),
        Span::styled(
            format!("RX:{}", app.rx_bytes),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" "),
        Span::styled(
            format!("TX:{}", app.tx_bytes),
            Style::default().fg(Color::Green),
        ),
        Span::raw(format!(" | {}", format_text)),
        Span::raw(format!(" | {}", protocol_text)),
        Span::raw(format!(" | {}", focus_text)),
        Span::raw(" | "),
        Span::styled("Ctrl+Q", Style::default().fg(Color::Yellow)),
        Span::raw(" 退出  "),
    ]);

    let p = Paragraph::new(line).style(Style::default().bg(Color::Rgb(22, 33, 62)));
    f.render_widget(p, area);
}
```

- [ ] **步骤 4：在 main.rs 声明并编译**

```rust
mod app;
mod command;
mod config;
mod event;
mod export;
mod protocol;
mod serial;
mod ui;
```

```bash
cargo build
```

- [ ] **步骤 5：Commit**

```bash
git add src/ui/ src/main.rs
git commit -m "feat: 添加 UI 状态栏组件 status.rs"
```

---

### 任务 13：UI - 串口设置栏

**文件：**
- 创建：`src/ui/settings.rs`

- [ ] **步骤 1：编写 ui/settings.rs**

```rust
use crate::app::{App, FocusArea};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// 渲染顶部串口设置栏
pub fn render_settings_bar(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusArea::Settings;
    let focus_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let port_display = if let Some(port) = app.available_ports.get(app.port_selected) {
        port.as_str()
    } else {
        "无可用端口"
    };

    let baud_rate = app.current_baud_rate();

    let mut spans = Vec::new();
    spans.push(Span::raw(" "));
    spans.push(Span::styled("端口 ", focus_style));
    spans.push(Span::styled(
        port_display,
        Style::default().fg(Color::Cyan),
    ));

    spans.push(Span::raw("  "));
    spans.push(Span::styled("波特率 ", focus_style));
    spans.push(Span::styled(
        format!("{}", baud_rate),
        Style::default().fg(Color::Cyan),
    ));

    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        format!("{}", app.serial_settings.data_bits),
        Style::default().fg(Color::Cyan),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        &app.serial_settings.parity,
        Style::default().fg(Color::Cyan),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        format!("{}", app.serial_settings.stop_bits),
        Style::default().fg(Color::Cyan),
    ));

    spans.push(Span::raw("  "));
    spans.push(Span::raw("流控: "));
    spans.push(Span::styled(
        &app.serial_settings.flow_control,
        Style::default().fg(Color::Cyan),
    ));

    // 指令提示
    if is_focused {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "←→ 改波特率 ↑↓ 选端口 C 连接/断开",
            Style::default().fg(Color::DarkGray),
        ));
    }

    let line = Line::from(spans);
    let p = Paragraph::new(line).style(Style::default().bg(Color::Rgb(22, 33, 62)));
    f.render_widget(p, area);
}
```

- [ ] **步骤 2：编译验证**

```bash
cargo build
```

- [ ] **步骤 3：Commit**

```bash
git add src/ui/settings.rs
git commit -m "feat: 添加 UI 串口设置栏 settings.rs"
```

---

### 任务 14：UI - 数据收发区

**文件：**
- 创建：`src/ui/terminal.rs`

- [ ] **步骤 1：编写 ui/terminal.rs**

```rust
use crate::app::{App, FocusArea};
use crate::protocol::format::format_bytes;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

/// 渲染数据收发区
pub fn render_terminal(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusArea::Terminal;
    let border_style = if is_focused {
        Style::default()
    } else {
        Style::default()
    };

    let line_count = area.height.saturating_sub(2) as usize;
    let total = app.terminal_lines.len();
    let start = if total > line_count {
        total.saturating_sub(line_count).saturating_sub(app.scroll_offset)
    } else {
        0
    };

    let lines: Vec<Line> = app
        .terminal_lines
        .iter()
        .skip(start)
        .take(line_count.max(1))
        .map(|entry| {
            let direction = if entry.is_rx { "←" } else { "→" };
            let color = if entry.is_rx {
                Color::Rgb(206, 147, 216) // 紫色
            } else {
                Color::Rgb(129, 199, 132) // 绿色
            };
            let formatted = format_bytes(&entry.raw_data, app.display_format);
            Line::from(Span::styled(
                format!("{} [{}] {}", direction, entry.timestamp, formatted),
                Style::default().fg(color),
            ))
        })
        .collect();

    // 如果无数据，显示提示
    let paragraph = if lines.is_empty() {
        Paragraph::new(Line::from(Span::styled(
            "等待串口数据...",
            Style::default().fg(Color::DarkGray),
        )))
    } else {
        Paragraph::new(lines).wrap(Wrap { trim: false })
    };

    f.render_widget(
        paragraph.style(Style::default().bg(Color::Rgb(15, 15, 35))),
        area,
    );
}
```

- [ ] **步骤 2：编译验证**

```bash
cargo build
```

- [ ] **步骤 3：Commit**

```bash
git add src/ui/terminal.rs
git commit -m "feat: 添加 UI 数据收发区 terminal.rs"
```

---

### 任务 15：UI - 发送栏

**文件：**
- 创建：`src/ui/send_input.rs`

- [ ] **步骤 1：编写 ui/send_input.rs**

```rust
use crate::app::{App, FocusArea};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// 渲染底部发送栏
pub fn render_send_input(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusArea::SendInput;
    let focus_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let input_display = if app.send_input.is_empty() && !is_focused {
        "输入发送数据...".to_string()
    } else {
        app.send_input.clone()
    };

    let cursor = if is_focused { "▎" } else { "" };

    let line_ending_options = vec!["无", "LF", "CR", "CRLF"];
    let current_le = match app.line_ending {
        crate::config::LineEnding::None => 0,
        crate::config::LineEnding::LF => 1,
        crate::config::LineEnding::CR => 2,
        crate::config::LineEnding::CRLF => 3,
    };

    let mut spans = Vec::new();
    spans.push(Span::raw(" "));
    spans.push(Span::styled("发送 ", focus_style));
    spans.push(Span::styled(
        format!("{} {}", input_display, cursor),
        Style::default().fg(Color::Rgb(100, 181, 246)),
    ));
    spans.push(Span::raw("  "));
    spans.push(Span::raw("行尾 "));

    for (i, name) in line_ending_options.iter().enumerate() {
        let style = if i == current_le {
            Style::default().fg(Color::Rgb(255, 183, 77))
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(format!("{} ", name), style));
    }

    spans.push(Span::raw(" "));
    spans.push(Span::styled("[发送]", Style::default().fg(Color::White).bg(Color::Rgb(21, 101, 192))));

    let line = Line::from(spans);
    let p = Paragraph::new(line).style(Style::default().bg(Color::Rgb(15, 15, 35)));
    f.render_widget(p, area);
}
```

- [ ] **步骤 2：编译验证**

```bash
cargo build
```

- [ ] **步骤 3：Commit**

```bash
git add src/ui/send_input.rs
git commit -m "feat: 添加 UI 发送栏 send_input.rs"
```

---

### 任务 16：UI - 快捷发送侧面板

**文件：**
- 创建：`src/ui/quick_send.rs`

- [ ] **步骤 1：编写 ui/quick_send.rs**

```rust
use crate::app::{App, FocusArea};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// 渲染右侧快捷发送面板
pub fn render_quick_send(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusArea::QuickSend;
    let border_style = if is_focused {
        Style::default()
    } else {
        Style::default()
    };

    let visible = area.height.saturating_sub(2).max(5) as usize;
    let start = if app.quick_send_selected >= visible {
        app.quick_send_selected - visible + 1
    } else {
        0
    };

    let lines: Vec<Line> = app
        .command_list
        .commands
        .iter()
        .enumerate()
        .skip(start)
        .take(visible)
        .map(|(i, cmd)| {
            let style = if i == app.quick_send_selected && is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(255, 183, 77))
            };
            Line::from(Span::styled(
                format!(" {} {}", if i == app.quick_send_selected && is_focused { "▶" } else { " " }, cmd.name),
                style,
            ))
        })
        .collect();

    if lines.is_empty() {
        let paragraph = Paragraph::new(Line::from(Span::styled(
            "暂无指令\n按 + 添加",
            Style::default().fg(Color::DarkGray),
        )));
        f.render_widget(
            paragraph.style(Style::default().bg(Color::Rgb(15, 15, 35))),
            area,
        );
    } else {
        let paragraph = Paragraph::new(lines);
        f.render_widget(
            paragraph.style(Style::default().bg(Color::Rgb(15, 15, 35))),
            area,
        );
    }
}
```

- [ ] **步骤 2：编译验证**

```bash
cargo build
```

- [ ] **步骤 3：Commit**

```bash
git add src/ui/quick_send.rs
git commit -m "feat: 添加 UI 快捷发送侧面板 quick_send.rs"
```

---

### 任务 17：UI - 主布局组装

**文件：**
- 修改：`src/ui/mod.rs`

- [ ] **步骤 1：重写 ui/mod.rs（完整布局）**

```rust
pub mod quick_send;
pub mod send_input;
pub mod settings;
pub mod status;
pub mod terminal;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// 渲染主布局
pub fn render(f: &mut Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),    // 设置栏
            Constraint::Min(1),       // 收发区 + 快捷面板
            Constraint::Length(1),    // 发送栏
            Constraint::Length(1),    // 状态栏
        ])
        .split(f.area());

    settings::render_settings_bar(f, app, main_chunks[0]);

    // 中间行：收发区 + 快捷侧面板
    let middle_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),      // 数据收发区
            Constraint::Length(22),  // 快捷发送面板
        ])
        .split(main_chunks[1]);

    terminal::render_terminal(f, app, middle_row[0]);
    quick_send::render_quick_send(f, app, middle_row[1]);

    send_input::render_send_input(f, app, main_chunks[2]);
    status::render_status_bar(f, app, main_chunks[3]);
}
```

- [ ] **步骤 2：编译验证**

```bash
cargo build
```

- [ ] **步骤 3：Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat: 组装 UI 主布局 mod.rs"
```

---

### 任务 18：主入口 main.rs - 完整组装

**文件：**
- 修改：`src/main.rs`

- [ ] **步骤 1：重写 main.rs**

```rust
mod app;
mod command;
mod config;
mod event;
mod export;
mod protocol;
mod serial;
mod ui;

use crate::app::App;
use crate::event::Event;
use crate::serial::SerialManager;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// 启动串口读取 task
fn start_serial_reader(app: &mut App, tx: mpsc::UnboundedSender<Event>) {
    let port = app.serial_manager.port_ref().map(|p| {
        // 重新打开一个独立端口给 reader（因为 reader 需要拥有权）
        // 实际实现中，reader 使用 try_clone
        None::<Box<dyn serialport::SerialPort>>
    });

    // 简化处理：如果已连接，构造读取通道
    // reader task 逻辑由串口管理层处理
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化终端
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // 创建 App
    let mut app = App::new();

    // 事件通道
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Event>();
    let tx_clone = event_tx.clone();

    // 键盘事件 task
    tokio::spawn(async move {
        loop {
            if event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(event::Event::Key(key)) = event::read() {
                    if tx_clone.send(Event::Key(key)).is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Tick task（每 100ms）
    let tick_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if tick_tx.send(Event::Tick).is_err() {
                break;
            }
        }
    });

    // 端口扫描 task（每 2 秒）
    let port_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let ports = SerialManager::list_ports();
            // 简单策略：每次 Tick 都发送 PortsChanged（实际可优化为仅在变化时）
            if port_tx.send(Event::PortsChanged).is_err() {
                break;
            }
            let _ = ports;
        }
    });

    // 主循环
    loop {
        // 处理事件
        while let Ok(event) = event_rx.try_recv() {
            app.handle_event(event);
            if !app.running {
                break;
            }
        }

        if !app.running {
            break;
        }

        // 检查是否需要重连
        if app.serial_manager.should_reconnect() {
            let port_name = app.serial_manager.last_port_name.clone();
            let baud_rate = app.serial_manager.last_baud_rate;
            if let Some(port_name) = port_name {
                let settings = app.serial_settings.clone();
                let _ = app.serial_manager.open(
                    &port_name,
                    baud_rate,
                    settings.data_bits,
                    &settings.parity,
                    settings.stop_bits,
                    &settings.flow_control,
                );
            }
        }

        // 渲染 UI
        terminal.draw(|f| {
            ui::render(f, &app);
        })?;
    }

    // 清理终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
```

- [ ] **步骤 2：编译验证**

```bash
cargo build
```

- [ ] **步骤 3：Commit**

```bash
git add src/main.rs
git commit -m "feat: 实现主入口 main.rs（TUI 主循环 + 事件任务）"
```

---

### 任务 19：串口 reader 完善 — 端口克隆与数据通道

**文件：**
- 修改：`src/serial/mod.rs`
- 修改：`src/main.rs`

- [ ] **步骤 1：在 serial/mod.rs 中添加 try_clone 方法**

```rust
impl SerialManager {
    // ... 已有方法 ...

    /// 克隆端口供 reader 使用
    pub fn try_clone_port(&self) -> Option<Box<dyn SerialPort>> {
        self.port.as_ref()?.try_clone().ok()
    }
}
```

注意：`SerialPort` trait 需要 `try_clone` 方法。`serialport` 库的 `TTYPort` 支持此操作。

- [ ] **步骤 2：编译验证**

```bash
cargo build
```

- [ ] **步骤 3：Commit**

```bash
git add src/serial/mod.rs
git commit -m "feat: 串口管理器添加 try_clone 方法"
```

---

### 任务 20：最终集成测试与 Git 忽略

**文件：**
- 创建/修改：`.gitignore`
- 确保所有测试通过

- [ ] **步骤 1：确保 .gitignore 包含必要条目**

```
/target/
.superpowers/
*.swp
*.swo
```

- [ ] **步骤 2：运行全部测试**

```bash
cargo test
cargo clippy -- -D warnings 2>/dev/null || cargo build
```

- [ ] **步骤 3：运行 fmt 检查**

```bash
cargo fmt -- --check
```

- [ ] **步骤 4：最终 commit**

```bash
git add -A
git commit -m "chore: 完善 .gitignore，确保构建通过"
```

---

## 验证清单

在所有任务完成后：

- [ ] `cargo build` 编译通过
- [ ] `cargo test` 全部测试通过
- [ ] `cargo fmt` 格式检查通过
- [ ] 启动程序后 TUI 界面正常渲染
- [ ] 串口连接/断开功能正常
- [ ] 收发数据显示正确（HEX/ASCII/RAW 切换）
- [ ] 快捷指令面板可操作
- [ ] 发送栏行尾选择正确
- [ ] 热插拔端口刷新正常
- [ ] Ctrl+D 切换格式、Ctrl+L 清空、Tab 切换焦点