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
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut app = App::new();

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Event>();
    let tx_clone = event_tx.clone();

    // 键盘事件 task
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
            if port_tx.send(Event::PortsChanged).is_err() {
                break;
            }
        }
    });

    // 串口写入通道：App → 主循环 → 串口
    let (write_tx, mut write_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    app.set_write_tx(write_tx);

    // reader task 管理
    let mut reader_handle: Option<tokio::task::JoinHandle<()>> = None;
    let mut reader_running: Option<Arc<Mutex<bool>>> = None;

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

        // 处理串口写入：消费 write_rx channel
        while let Ok(data) = write_rx.try_recv() {
            if let Some(port) = app.serial_manager.port_mut() {
                let _ = port.write(&data);
            }
        }

        // 连接后启动 reader
        if app.serial_manager.is_connected() && reader_handle.is_none() {
            if let Some(cloned_port) = app.serial_manager.try_clone_port() {
                let running = Arc::new(Mutex::new(true));
                reader_running = Some(running.clone());
                let tx = event_tx.clone();
                reader_handle = Some(crate::serial::reader::start_reader_task(
                    cloned_port,
                    tx,
                    running,
                ));
            }
        }

        // 断开后停止 reader
        if !app.serial_manager.is_connected() && reader_handle.is_some() {
            if let Some(running) = reader_running.take() {
                *running.lock().unwrap() = false;
            }
            if let Some(handle) = reader_handle.take() {
                handle.abort();
            }
        }

        // 重连逻辑
        if app.serial_manager.should_reconnect() {
            let port_name = app.serial_manager.last_port_name.clone();
            let baud_rate = app.serial_manager.last_baud_rate;
            if let Some(ref port_name) = port_name {
                let settings = app.serial_settings.clone();
                let _ = app.serial_manager.open(
                    port_name,
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

    // 清理：停止 reader
    if let Some(running) = reader_running {
        *running.lock().unwrap() = false;
    }
    if let Some(handle) = reader_handle {
        handle.abort();
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
