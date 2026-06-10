use anyhow::{Context, Result};
use std::io::Write;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataRecord {
    pub time: String,
    pub dir: String,
    pub data: String,
}

#[allow(dead_code)]
pub fn export_csv(records: &[DataRecord], path: &str) -> Result<()> {
    let mut file = std::fs::File::create(path)
        .with_context(|| format!("\u{65e0}\u{6cd5}\u{521b}\u{5efa}\u{6587}\u{4ef6} {}", path))?;
    writeln!(
        file,
        "\u{65f6}\u{95f4}\u{6233},\u{65b9}\u{5411},\u{6570}\u{636e}"
    )?;
    for r in records {
        writeln!(file, "{},{},{}", r.time, r.dir, r.data)?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn export_json(records: &[DataRecord], path: &str) -> Result<()> {
    let content = serde_json::to_string_pretty(records)?;
    std::fs::write(path, content)
        .with_context(|| format!("\u{65e0}\u{6cd5}\u{5199}\u{5165}\u{6587}\u{4ef6} {}", path))?;
    Ok(())
}

#[allow(dead_code)]
pub fn export_text(records: &[DataRecord], path: &str) -> Result<()> {
    let mut file = std::fs::File::create(path)
        .with_context(|| format!("\u{65e0}\u{6cd5}\u{521b}\u{5efa}\u{6587}\u{4ef6} {}", path))?;
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
