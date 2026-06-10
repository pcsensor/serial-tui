use crate::command::CommandList;
use crate::config::{AppConfig, DisplayFormat, LineEnding, SerialSettings};
use crate::event::Event;
use crate::export::DataRecord;
use crate::protocol::parser::ParserRegistry;
use crate::serial::{ConnectionState, SerialManager};
use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    Settings,
    Terminal,
    SendInput,
    QuickSend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickSendMode {
    Normal,
    Adding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Txt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportField {
    Format,
    Dir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportDialogState {
    Hidden,
    Open {
        format: ExportFormat,
        dir: String,
        field: ExportField,
    },
}

#[derive(Debug, Clone)]
pub struct TerminalLine {
    pub timestamp: String,
    pub is_rx: bool,
    pub raw_data: Vec<u8>,
}

pub struct App {
    pub serial_manager: SerialManager,
    pub terminal_lines: Vec<TerminalLine>,
    pub data_records: Vec<DataRecord>,
    pub scroll_offset: usize,
    pub focus: FocusArea,
    /// 接收数据缓冲区（用于按 \n 分行）
    pub rx_buffer: Vec<u8>,
    pub serial_settings: SerialSettings,
    pub display_format: DisplayFormat,
    pub line_ending: LineEnding,
    pub send_input: String,
    pub command_list: CommandList,
    pub quick_send_selected: usize,
    pub quick_send_mode: QuickSendMode,
    pub available_ports: Vec<String>,
    pub port_selected: usize,
    pub baud_rate_selected: usize,
    /// 设置栏子项索引：0=端口,1=波特率,2=数据位,3=校验位,4=停止位,5=流控
    pub settings_sub_index: usize,
    /// 数据位选项
    pub data_bits_options: Vec<u8>,
    pub data_bits_selected: usize,
    /// 校验位选项
    pub parity_options: Vec<&'static str>,
    pub parity_selected: usize,
    /// 停止位选项
    pub stop_bits_options: Vec<u8>,
    pub stop_bits_selected: usize,
    /// 流控选项
    pub flow_control_options: Vec<&'static str>,
    pub flow_control_selected: usize,
    pub parser_registry: ParserRegistry,
    pub running: bool,
    pub config: AppConfig,
    pub rx_bytes: u64,
    pub tab_hint_ticks: u8,
    pub export_dialog: ExportDialogState,
    pub tx_bytes: u64,
    pub status_message: String,
    write_tx: Option<UnboundedSender<Vec<u8>>>,
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
            rx_buffer: Vec::new(),
            serial_settings: config.serial.clone(),
            display_format: config.display_format,
            line_ending: config.line_ending,
            send_input: String::new(),
            command_list,
            quick_send_selected: 0,
            quick_send_mode: QuickSendMode::Normal,
            available_ports: SerialManager::list_ports(),
            port_selected: 0,
            baud_rate_selected: 6,
            settings_sub_index: 0,
            data_bits_options: vec![5, 6, 7, 8],
            data_bits_selected: 3, // 默认 8
            parity_options: vec!["none", "odd", "even"],
            parity_selected: 0, // 默认 none
            stop_bits_options: vec![1, 2],
            stop_bits_selected: 0, // 默认 1
            flow_control_options: vec!["none", "hardware", "software"],
            flow_control_selected: 0, // 默认 none
            parser_registry: ParserRegistry::default_parsers(),
            running: true,
            config,
            rx_bytes: 0,
            tab_hint_ticks: 0,
            export_dialog: ExportDialogState::Hidden,
            tx_bytes: 0,
            status_message: String::new(),
            write_tx: None,
            baud_rates: vec![
                300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
            ],
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::RxData(data) => self.handle_rx_data(data),
            Event::PortsChanged => {
                self.available_ports = SerialManager::list_ports();
            }
            Event::SerialError(msg) => {
                self.status_message = format!("\u{9519}\u{8bef}: {}", msg);
                self.serial_manager.enter_reconnect();
            }
            Event::Tick => {
                if self.tab_hint_ticks > 0 {
                    self.tab_hint_ticks -= 1;
                }
                if self.serial_manager.should_reconnect() {
                    self.status_message = format!(
                        "\u{6b63}\u{5728}\u{91cd}\u{8fde} {}  (\u{7b2c} {} \u{6b21})...",
                        self.serial_manager.last_port_name.as_deref().unwrap_or("?"),
                        self.serial_manager.reconnect_count
                    );
                }
            }
            Event::Quit => {
                self.save_config();
                self.command_list.save().ok();
                self.running = false;
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
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
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.export_data();
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
                self.tab_hint_ticks = 20;
                return;
            }
            _ => {}
        }

        match self.focus {
            FocusArea::Settings => self.handle_settings_key(key),
            FocusArea::Terminal => self.handle_terminal_key(key),
            FocusArea::SendInput => self.handle_send_input_key(key),
            FocusArea::QuickSend => self.handle_quick_send_key(key),
        }
    }

    fn handle_rx_data(&mut self, data: Vec<u8>) {
        self.rx_bytes += data.len() as u64;

        // 追加到缓冲区
        self.rx_buffer.extend_from_slice(&data);

        // 按 \n 分割处理
        while let Some(pos) = self.rx_buffer.iter().position(|&b| b == b'\n') {
            let line_data: Vec<u8> = self.rx_buffer.drain(..=pos).collect();
            // 移除末尾的 \n（和可能的 \r）
            let trimmed = if line_data.len() >= 2
                && line_data[line_data.len() - 2] == b'\r'
                && line_data[line_data.len() - 1] == b'\n'
            {
                &line_data[..line_data.len() - 2]
            } else if line_data.last() == Some(&b'\n') {
                &line_data[..line_data.len() - 1]
            } else {
                &line_data
            };

            if trimmed.is_empty() {
                continue;
            }

            let now = Local::now();
            let timestamp = now.format("%H:%M:%S.%3f").to_string();

            let formatted = crate::protocol::format::format_bytes(trimmed, self.display_format);
            let display = self
                .parser_registry
                .parse(trimmed)
                .unwrap_or(formatted.clone());

            self.terminal_lines.push(TerminalLine {
                timestamp: timestamp.clone(),
                is_rx: true,
                raw_data: trimmed.to_vec(),
            });

            self.data_records.push(DataRecord {
                time: timestamp,
                dir: "RX".into(),
                data: display,
            });
        }

        // 如果缓冲区超过 4KB 且没有换行，强制显示
        if self.rx_buffer.len() > 4096 {
            let now = Local::now();
            let timestamp = now.format("%H:%M:%S.%3f").to_string();
            let data = std::mem::take(&mut self.rx_buffer);

            let formatted = crate::protocol::format::format_bytes(&data, self.display_format);
            let display = self
                .parser_registry
                .parse(&data)
                .unwrap_or(formatted.clone());

            self.terminal_lines.push(TerminalLine {
                timestamp: timestamp.clone(),
                is_rx: true,
                raw_data: data,
            });

            self.data_records.push(DataRecord {
                time: timestamp,
                dir: "RX".into(),
                data: display,
            });
        }

        if self.scroll_offset > 0 {
            self.scroll_offset = 0;
        }
    }

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

    pub fn set_write_tx(&mut self, tx: UnboundedSender<Vec<u8>>) {
        self.write_tx = Some(tx);
    }

    #[allow(dead_code)]
    pub fn write_tx_clone(&self) -> Option<UnboundedSender<Vec<u8>>> {
        self.write_tx.clone()
    }

    fn save_config(&mut self) {
        self.config.serial = self.serial_settings.clone();
        self.config.line_ending = self.line_ending;
        self.config.display_format = self.display_format;
        self.config.save().ok();
    }

    fn export_data(&mut self) {
        if self.data_records.is_empty() {
            self.status_message = "\u{65e0}\u{6570}\u{636e}\u{53ef}\u{5bfc}\u{51fa}".to_string();
            return;
        }

        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("serial_export_{}.csv", timestamp);

        match crate::export::export_csv(&self.data_records, &filename) {
            Ok(_) => {
                self.status_message = format!("\u{5df2}\u{5bfc}\u{51fa}: {}", filename);
            }
            Err(e) => {
                self.status_message = format!("\u{5bfc}\u{51fa}\u{5931}\u{8d25}: {}", e);
            }
        }
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        // 字母快捷键选择设置项
        match key.code {
            KeyCode::Char('p') => {
                self.settings_sub_index = 0;
                return;
            }
            KeyCode::Char('b') => {
                self.settings_sub_index = 1;
                return;
            }
            KeyCode::Char('d') => {
                self.settings_sub_index = 2;
                return;
            }
            KeyCode::Char('y') => {
                self.settings_sub_index = 3;
                return;
            }
            KeyCode::Char('s') => {
                self.settings_sub_index = 4;
                return;
            }
            KeyCode::Char('f') => {
                self.settings_sub_index = 5;
                return;
            }
            KeyCode::Char('c') => {
                self.toggle_connection();
                return;
            }
            _ => {}
        }

        // ↑↓ 改变当前选中项的值
        match self.settings_sub_index {
            0 => {
                // 端口
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
                    _ => {}
                }
            }
            1 => {
                // 波特率
                match key.code {
                    KeyCode::Up => {
                        if self.baud_rate_selected < self.baud_rates.len() - 1 {
                            self.baud_rate_selected += 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.baud_rate_selected > 0 {
                            self.baud_rate_selected -= 1;
                        }
                    }
                    _ => {}
                }
            }
            2 => {
                // 数据位
                match key.code {
                    KeyCode::Up => {
                        if self.data_bits_selected < self.data_bits_options.len() - 1 {
                            self.data_bits_selected += 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.data_bits_selected > 0 {
                            self.data_bits_selected -= 1;
                        }
                    }
                    _ => {}
                }
            }
            3 => {
                // 校验位
                match key.code {
                    KeyCode::Up => {
                        if self.parity_selected < self.parity_options.len() - 1 {
                            self.parity_selected += 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.parity_selected > 0 {
                            self.parity_selected -= 1;
                        }
                    }
                    _ => {}
                }
            }
            4 => {
                // 停止位
                match key.code {
                    KeyCode::Up => {
                        if self.stop_bits_selected < self.stop_bits_options.len() - 1 {
                            self.stop_bits_selected += 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.stop_bits_selected > 0 {
                            self.stop_bits_selected -= 1;
                        }
                    }
                    _ => {}
                }
            }
            5 => {
                // 流控
                match key.code {
                    KeyCode::Up => {
                        if self.flow_control_selected < self.flow_control_options.len() - 1 {
                            self.flow_control_selected += 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.flow_control_selected > 0 {
                            self.flow_control_selected -= 1;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        self.sync_settings();
    }

    /// 将选中的选项同步到 serial_settings
    fn sync_settings(&mut self) {
        if let Some(port) = self.available_ports.get(self.port_selected) {
            self.serial_settings.port = port.clone();
        }
        self.serial_settings.baud_rate = self.current_baud_rate();
        self.serial_settings.data_bits = self.data_bits_options[self.data_bits_selected];
        self.serial_settings.parity = self.parity_options[self.parity_selected].to_string();
        self.serial_settings.stop_bits = self.stop_bits_options[self.stop_bits_selected];
        self.serial_settings.flow_control =
            self.flow_control_options[self.flow_control_selected].to_string();
    }

    pub fn toggle_connection(&mut self) {
        match self.serial_manager.state {
            ConnectionState::Connected => {
                self.serial_manager.disconnect_by_user();
            }
            _ => {
                if let Some(port_name) = self.available_ports.get(self.port_selected).cloned() {
                    let baud_rate = self
                        .baud_rates
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
                            self.status_message = format!("\u{5df2}\u{8fde}\u{63a5} {}", port_name);
                        }
                        Err(e) => {
                            self.status_message =
                                format!("\u{8fde}\u{63a5}\u{5931}\u{8d25}: {}", e);
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
            KeyCode::Char('l') if key.modifiers.is_empty() => {
                // 循环切换行尾
                self.line_ending = match self.line_ending {
                    LineEnding::None => LineEnding::LF,
                    LineEnding::LF => LineEnding::CR,
                    LineEnding::CR => LineEnding::CRLF,
                    LineEnding::CRLF => LineEnding::None,
                };
            }
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
            _ => {}
        }
    }

    fn handle_quick_send_key(&mut self, key: KeyEvent) {
        match self.quick_send_mode {
            QuickSendMode::Adding => {
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
                            let name = if input.len() > 20 {
                                input[..20].to_string()
                            } else {
                                input.clone()
                            };
                            self.command_list.add(name, input);
                            self.command_list.save().ok();
                            self.quick_send_mode = QuickSendMode::Normal;
                        }
                    }
                    KeyCode::Esc => {
                        self.send_input.clear();
                        self.quick_send_mode = QuickSendMode::Normal;
                    }
                    _ => {}
                }
                return;
            }
            QuickSendMode::Normal => {}
        }

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
            KeyCode::Char('a') => {
                self.send_input.clear();
                self.quick_send_mode = QuickSendMode::Adding;
            }
            KeyCode::Char('d') => {
                if !self.command_list.commands.is_empty() {
                    self.command_list.remove(self.quick_send_selected);
                    if self.quick_send_selected >= self.command_list.len()
                        && self.quick_send_selected > 0
                    {
                        self.quick_send_selected -= 1;
                    }
                    self.command_list.save().ok();
                }
            }
            _ => {}
        }
    }

    pub fn current_baud_rate(&self) -> u32 {
        self.baud_rates
            .get(self.baud_rate_selected)
            .copied()
            .unwrap_or(115200)
    }

    #[allow(dead_code)]
    pub fn baud_rates(&self) -> &[u32] {
        &self.baud_rates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_tab_hint_ticks_is_zero() {
        let app = App::new();
        assert_eq!(app.tab_hint_ticks, 0);
    }

    #[test]
    fn test_initial_export_dialog_is_hidden() {
        let app = App::new();
        assert!(matches!(app.export_dialog, ExportDialogState::Hidden));
    }

    #[test]
    fn test_tab_sets_hint_ticks() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mut app = App::new();
        // App::new() 初始 focus 为 Terminal
        assert_eq!(app.focus, FocusArea::Terminal);
        app.tab_hint_ticks = 0;
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
        assert_eq!(app.tab_hint_ticks, 20);
        // Tab 后 Terminal → SendInput
        assert_eq!(app.focus, FocusArea::SendInput);
    }

    #[test]
    fn test_tick_decrements_hint_ticks() {
        let mut app = App::new();
        app.tab_hint_ticks = 5;
        app.handle_event(Event::Tick);
        assert_eq!(app.tab_hint_ticks, 4);
    }

    #[test]
    fn test_tick_does_not_underflow() {
        let mut app = App::new();
        app.tab_hint_ticks = 0;
        app.handle_event(Event::Tick);
        assert_eq!(app.tab_hint_ticks, 0);
    }
}
