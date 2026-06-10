pub mod reader;
pub mod writer;

use anyhow::{Context, Result};
use serialport::{available_ports, SerialPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connected,
    Reconnecting,
}

pub struct SerialManager {
    port: Option<Box<dyn SerialPort>>,
    pub state: ConnectionState,
    pub reconnect_count: u32,
    pub last_port_name: Option<String>,
    pub last_baud_rate: u32,
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

    pub fn list_ports() -> Vec<String> {
        available_ports()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.port_name)
            .collect()
    }

    pub fn open(
        &mut self,
        port_name: &str,
        baud_rate: u32,
        data_bits: u8,
        parity: &str,
        stop_bits: u8,
        flow_control: &str,
    ) -> Result<()> {
        let builder = serialport::new(port_name, baud_rate);

        let builder = builder.data_bits(match data_bits {
            5 => serialport::DataBits::Five,
            6 => serialport::DataBits::Six,
            7 => serialport::DataBits::Seven,
            _ => serialport::DataBits::Eight,
        });

        let builder = builder.parity(match parity {
            "odd" => serialport::Parity::Odd,
            "even" => serialport::Parity::Even,
            _ => serialport::Parity::None,
        });

        let builder = builder.stop_bits(match stop_bits {
            2 => serialport::StopBits::Two,
            _ => serialport::StopBits::One,
        });

        let builder = builder.flow_control(match flow_control {
            "hardware" => serialport::FlowControl::Hardware,
            "software" => serialport::FlowControl::Software,
            _ => serialport::FlowControl::None,
        });

        let port = builder.open().with_context(|| {
            format!(
                "\u{65e0}\u{6cd5}\u{6253}\u{5f00}\u{4e32}\u{53e3} {}",
                port_name
            )
        })?;

        self.port = Some(port);
        self.state = ConnectionState::Connected;
        self.last_port_name = Some(port_name.to_string());
        self.last_baud_rate = baud_rate;
        self.reconnect_count = 0;
        self.user_disconnect = false;

        Ok(())
    }

    pub fn close(&mut self) {
        self.port = None;
        self.state = ConnectionState::Disconnected;
    }

    pub fn disconnect_by_user(&mut self) {
        self.user_disconnect = true;
        self.close();
    }

    pub fn enter_reconnect(&mut self) {
        if !self.user_disconnect {
            self.port = None;
            self.state = ConnectionState::Reconnecting;
            self.reconnect_count += 1;
        }
    }

    pub fn should_reconnect(&self) -> bool {
        self.state == ConnectionState::Reconnecting
            && !self.user_disconnect
            && self.reconnect_count <= 10
            && self.last_port_name.is_some()
    }

    pub fn port_ref(&self) -> Option<&dyn SerialPort> {
        self.port.as_ref().map(|p| p.as_ref())
    }

    pub fn port_mut(&mut self) -> Option<&mut Box<dyn SerialPort>> {
        self.port.as_mut()
    }

    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    pub fn try_clone_port(&self) -> Option<Box<dyn SerialPort>> {
        self.port.as_ref()?.try_clone().ok()
    }
}
