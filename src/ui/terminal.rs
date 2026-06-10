use crate::app::{App, FocusArea};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

pub fn render_terminal(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusArea::Terminal;

    let line_count = area.height.saturating_sub(2) as usize;
    let total = app.terminal_lines.len();
    let start = if total > line_count {
        total
            .saturating_sub(line_count)
            .saturating_sub(app.scroll_offset)
    } else {
        0
    };

    let lines: Vec<Line> = app
        .terminal_lines
        .iter()
        .skip(start)
        .take(line_count.max(1))
        .map(|entry| {
            let direction = if entry.is_rx { "\u{2190}" } else { "\u{2192}" };
            let color = if entry.is_rx {
                Color::Rgb(206, 147, 216)
            } else {
                Color::Rgb(129, 199, 132)
            };
            let formatted =
                crate::protocol::format::format_bytes(&entry.raw_data, app.display_format);
            Line::from(Span::styled(
                format!("{} [{}] {}", direction, entry.timestamp, formatted),
                Style::default().fg(color),
            ))
        })
        .collect();

    let paragraph = if lines.is_empty() {
        Paragraph::new(Line::from(Span::styled(
            "\u{7b49}\u{5f85}\u{4e32}\u{53e3}\u{6570}\u{636e}... (\u{6309} \u{2191}\u{2193} \u{6eda}\u{52a8})",
            Style::default().fg(Color::DarkGray),
        )))
    } else {
        Paragraph::new(lines).wrap(Wrap { trim: false })
    };

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    f.render_widget(
        paragraph
            .style(Style::default().bg(Color::Rgb(15, 15, 35)))
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::NONE)
                    .style(border_style),
            ),
        area,
    );
}
