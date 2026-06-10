use crate::app::{App, FocusArea};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_settings_bar(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusArea::Settings;
    let focus_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let port_display = if let Some(port) = app.available_ports.get(app.port_selected) {
        port.as_str()
    } else {
        "\u{65e0}\u{53ef}\u{7528}\u{7aef}\u{53e3}"
    };

    let baud_rate = app.current_baud_rate();

    let mut spans = Vec::new();
    spans.push(Span::raw(" "));
    spans.push(Span::styled("\u{7aef}\u{53e3} ", focus_style));
    spans.push(Span::styled(port_display, Style::default().fg(Color::Cyan)));

    spans.push(Span::raw("  "));
    spans.push(Span::styled("\u{6ce2}\u{7279}\u{7387} ", focus_style));
    spans.push(Span::styled(
        format!("{}", baud_rate),
        Style::default().fg(Color::Cyan),
    ));

    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        format!("{}", app.serial_settings.data_bits),
        Style::default().fg(Color::Cyan),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        &app.serial_settings.parity,
        Style::default().fg(Color::Cyan),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        format!("{}", app.serial_settings.stop_bits),
        Style::default().fg(Color::Cyan),
    ));

    spans.push(Span::raw("  "));
    spans.push(Span::raw("\u{6d41}\u{63a7}: "));
    spans.push(Span::styled(
        &app.serial_settings.flow_control,
        Style::default().fg(Color::Cyan),
    ));

    if is_focused {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "\u{2190}\u{2192} \u{6ce2}\u{7279}\u{7387} \u{2191}\u{2193} \u{7aef}\u{53e3} C \u{8fde}\u{63a5}/\u{65ad}\u{5f00}",
            Style::default().fg(Color::DarkGray),
        ));
    }

    let line = Line::from(spans);
    let p = Paragraph::new(line).style(Style::default().bg(Color::Rgb(22, 33, 62)));
    f.render_widget(p, area);
}
