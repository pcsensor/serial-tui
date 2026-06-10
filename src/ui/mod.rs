pub mod quick_send;
pub mod send_input;
pub mod settings;
pub mod status;
pub mod terminal;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    settings::render_settings_bar(f, app, main_chunks[0]);

    let middle_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(22)])
        .split(main_chunks[1]);

    terminal::render_terminal(f, app, middle_row[0]);
    quick_send::render_quick_send(f, app, middle_row[1]);

    send_input::render_send_input(f, app, main_chunks[2]);
    status::render_status_bar(f, app, main_chunks[3]);
}
