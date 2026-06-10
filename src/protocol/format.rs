use crate::config::DisplayFormat;

pub fn format_bytes(data: &[u8], format: DisplayFormat) -> String {
    match format {
        DisplayFormat::Hex => data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" "),
        DisplayFormat::Ascii => data
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect(),
        DisplayFormat::Raw => String::from_utf8_lossy(data).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_format() {
        assert_eq!(format_bytes(b"Hello", DisplayFormat::Hex), "48 65 6C 6C 6F");
    }

    #[test]
    fn test_hex_format_empty() {
        assert_eq!(format_bytes(b"", DisplayFormat::Hex), "");
    }

    #[test]
    fn test_ascii_format() {
        let result = format_bytes(b"Hello\x00World", DisplayFormat::Ascii);
        assert!(result.contains("Hello"));
        assert!(result.contains('.'));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_raw_format() {
        assert_eq!(
            format_bytes(b"Hello World", DisplayFormat::Raw),
            "Hello World"
        );
    }

    #[test]
    fn test_single_byte_hex() {
        assert_eq!(format_bytes(&[0xFF], DisplayFormat::Hex), "FF");
        assert_eq!(format_bytes(&[0x0A], DisplayFormat::Hex), "0A");
    }
}
