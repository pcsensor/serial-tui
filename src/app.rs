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
    pub serial_settings: SerialSettings,
    pub display_format: DisplayFormat,
    pub line_ending: LineEnding,
    pub send_input: String,
    pub command_list: CommandList,
    pub quick_send_selected: usize,
    pub available_ports: Vec<String>,
    pub port_selected: usize,
    pub baud_rate_selected: usize,
    pub parser_registry: ParserRegistry,
    pub running: bool,
    pub config: AppConfig,
    pub rx_bytes: u64,
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
            serial_settings: config.serial.clone(),
            display_format: config.display_format,
            line_ending: config.line_ending,
            send_input: String::new(),
            command_list,
            quick_send_selected: 0,
            available_ports: SerialManager::list_ports(),
            port_selected: 0,
            baud_rate_selected: 6,
            parser_registry: ParserRegistry::default_parsers(),
            running: true,
            config,
            rx_bytes: 0,
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

        match self.focus {
            FocusArea::Settings => self.handle_settings_key(key),
            FocusArea::Terminal => self.handle_terminal_key(key),
            FocusArea::SendInput => self.handle_send_input_key(key),
            FocusArea::QuickSend => self.handle_quick_send_key(key),
        }
    }

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

    pub fn write_tx_clone(&self) -> Option<UnboundedSender<Vec<u8>>> {
        self.write_tx.clone()
    }

    fn save_config(&mut self) {
        self.config.serial = self.serial_settings.clone();
        self.config.line_ending = self.line_ending;
        self.config.display_format = self.display_format;
        self.config.save().ok();
    }

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
                if key.modifiers == KeyModifiers::CONTROL
                    || key.modifiers == KeyModifiers::NONE =>
            {
                self.toggle_connection();
            }
            _ => {}
        }
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

    pub fn current_baud_rate(&self) -> u32 {
        self.baud_rates
            .get(self.baud_rate_selected)
            .copied()
            .unwrap_or(115200)
    }

    pub fn baud_rates(&self) -> &[u32] {
        &self.baud_rates
    }
}
