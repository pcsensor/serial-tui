use crate::event::Event;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

pub fn start_reader_task(
    mut port: Box<dyn serialport::SerialPort>,
    tx: UnboundedSender<Event>,
    running: Arc<Mutex<bool>>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        let mut buf = vec![0u8; 256];
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

            match port.bytes_to_read() {
                Ok(0) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Ok(_) => {}
                Err(_) => {
                    let _ = tx.send(Event::SerialError(
                        "\u{4e32}\u{53e3}\u{8bfb}\u{53d6}\u{9519}\u{8bef}".into(),
                    ));
                    break;
                }
            }

            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let data = buf[..n].to_vec();
                    if tx.send(Event::RxData(data)).is_err() {
                        break;
                    }
                }
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    continue;
                }
                Err(_) => {
                    let _ = tx.send(Event::SerialError(
                        "\u{4e32}\u{53e3}\u{8bfb}\u{53d6}\u{65ad}\u{5f00}".into(),
                    ));
                    break;
                }
            }
        }
    })
}
