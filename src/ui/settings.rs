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
    let sub = app.settings_sub_index;

    let hl = |idx: usize| -> Style {
        if is_focused && sub == idx {
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(50, 50, 80))
        } else {
            Style::default()
        }
    };

    let port_display = if let Some(port) = app.available_ports.get(app.port_selected) {
        port.as_str()
    } else {
        "\u{65e0}\u{53ef}\u{7528}\u{7aef}\u{53e3}"
    };

    let data_bits = app.data_bits_options[app.data_bits_selected];
    let parity = app.parity_options[app.parity_selected];
    let stop_bits = app.stop_bits_options[app.stop_bits_selected];
    let flow_control = app.flow_control_options[app.flow_control_selected];

    let mut spans = Vec::new();
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        format!(
            "{}\u{7aef}\u{53e3} ",
            if is_focused && sub == 0 {
                "\u{25b6}"
            } else {
                ""
            }
        ),
        hl(0),
    ));
    spans.push(Span::styled(port_display, hl(0)));

    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        format!(
            "{}\u{6ce2}\u{7279}\u{7387} ",
            if is_focused && sub == 1 {
                "\u{25b6}"
            } else {
                ""
            }
        ),
        hl(1),
    ));
    spans.push(Span::styled(format!("{}", app.current_baud_rate()), hl(1)));

    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        format!(
            "{}\u{6570}\u{636e}\u{4f4d} ",
            if is_focused && sub == 2 {
                "\u{25b6}"
            } else {
                ""
            }
        ),
        hl(2),
    ));
    spans.push(Span::styled(format!("{}", data_bits), hl(2)));

    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        format!(
            "{}\u{6821}\u{9a8c} ",
            if is_focused && sub == 3 {
                "\u{25b6}"
            } else {
                ""
            }
        ),
        hl(3),
    ));
    spans.push(Span::styled(parity, hl(3)));

    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        format!(
            "{}\u{505c}\u{6b62}\u{4f4d} ",
            if is_focused && sub == 4 {
                "\u{25b6}"
            } else {
                ""
            }
        ),
        hl(4),
    ));
    spans.push(Span::styled(format!("{}", stop_bits), hl(4)));

    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        format!(
            "{}\u{6d41}\u{63a7} ",
            if is_focused && sub == 5 {
                "\u{25b6}"
            } else {
                ""
            }
        ),
        hl(5),
    ));
    spans.push(Span::styled(flow_control, hl(5)));

    if is_focused {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "Tab切换 \u{2190}\u{2192}\u{6539}\u{503c} C\u{8fde}\u{63a5}",
            Style::default().fg(Color::DarkGray),
        ));
    }

    let line = Line::from(spans);
    let p = Paragraph::new(line).style(Style::default().bg(Color::Rgb(22, 33, 62)));
    f.render_widget(p, area);
}
