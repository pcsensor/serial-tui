use anyhow::Result;

#[allow(dead_code)]
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
