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
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut app = App::new();

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Event>();
    let tx_clone = event_tx.clone();

    tokio::spawn(async move {
        loop {
            if crossterm::event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                    if tx_clone.send(Event::Key(key)).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let tick_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if tick_tx.send(Event::Tick).is_err() {
                break;
            }
        }
    });

    let port_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            if port_tx.send(Event::PortsChanged).is_err() {
                break;
            }
        }
    });

    // 串口写入通道
    let (write_tx, mut write_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    app.set_write_tx(write_tx);

    // 串口写入任务
    let write_handle = tokio::spawn(async move {
        while let Some(data) = write_rx.recv().await {
            // 写入操作需要在 app 上下文完成
            // 这里只是占位，实际由 app.send_data 处理
            let _ = data;
        }
    });

    loop {
        while let Ok(event) = event_rx.try_recv() {
            app.handle_event(event);
            if !app.running {
                break;
            }
        }

        if !app.running {
            break;
        }

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

        terminal.draw(|f| {
            ui::render(f, &app);
        })?;
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
