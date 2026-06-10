pub trait ProtocolParser: Send + Sync {
    fn name(&self) -> &str;
    fn try_parse(&self, data: &[u8]) -> Option<String>;
}

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

pub struct JsonProtocol;

impl ProtocolParser for JsonProtocol {
    fn name(&self) -> &str {
        "JSON"
    }

    fn try_parse(&self, data: &[u8]) -> Option<String> {
        let s = String::from_utf8_lossy(data);
        let trimmed = s.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
                return serde_json::to_string_pretty(&value).ok();
            }
        }
        None
    }
}

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

pub struct ParserRegistry {
    parsers: Vec<Box<dyn ProtocolParser>>,
    pub active_index: usize,
}

impl ParserRegistry {
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

    pub fn active_name(&self) -> &str {
        self.parsers
            .get(self.active_index)
            .map(|p| p.name())
            .unwrap_or("\u{65e0}")
    }

    pub fn names(&self) -> Vec<String> {
        self.parsers.iter().map(|p| p.name().to_string()).collect()
    }

    pub fn next(&mut self) {
        if !self.parsers.is_empty() {
            self.active_index = (self.active_index + 1) % self.parsers.len();
        }
    }

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
        assert!(p.try_parse(&[0x01, 0x03]).is_none());
        let result = p.try_parse(&[0x01, 0x03, 0x00, 0x01]).unwrap();
        assert!(result.contains("\u{8bfb}\u{53d6}\u{4fdd}\u{6301}\u{5bc4}\u{5b58}\u{5668}"));
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
        assert_eq!(registry.names().len(), 3);
    }
}
