use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let state_text = match app.serial_manager.state {
        crate::serial::ConnectionState::Connected => Span::styled(
            "\u{25cf} \u{5df2}\u{8fde}\u{63a5}",
            Style::default().fg(Color::Green),
        ),
        crate::serial::ConnectionState::Disconnected => Span::styled(
            "\u{25cb} \u{672a}\u{8fde}\u{63a5}",
            Style::default().fg(Color::Gray),
        ),
        crate::serial::ConnectionState::Reconnecting => Span::styled(
            "\u{27f3} \u{91cd}\u{8fde}\u{4e2d}",
            Style::default().fg(Color::Yellow),
        ),
    };

    let format_text = format!("\u{683c}\u{5f0f}: {:?}", app.display_format);
    let protocol_text = format!("\u{534f}\u{8bae}: {}", app.parser_registry.active_name());

    let line = Line::from(vec![
        Span::raw(" "),
        state_text,
        Span::raw(" | "),
        Span::styled(
            format!("RX:{}", app.rx_bytes),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" "),
        Span::styled(
            format!("TX:{}", app.tx_bytes),
            Style::default().fg(Color::Green),
        ),
        Span::raw(format!(" | {}", format_text)),
        Span::raw(format!(" | {}", protocol_text)),
        Span::raw(" | "),
        Span::styled("Ctrl+Q", Style::default().fg(Color::Yellow)),
        Span::raw(" \u{9000}\u{51fa}  "),
    ]);

    let p = Paragraph::new(line).style(Style::default().bg(Color::Rgb(22, 33, 62)));
    f.render_widget(p, area);
}
