use crate::app::{App, FocusArea, QuickSendMode};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_send_input(f: &mut Frame, app: &App, area: Rect) {
    let is_adding = app.quick_send_mode == QuickSendMode::Adding;
    let is_focused = app.focus == FocusArea::SendInput || is_adding;
    let focus_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let label = if is_adding {
        "\u{6dfb}\u{52a0}\u{6307}\u{4ee4} "
    } else {
        "\u{53d1}\u{9001} "
    };

    let placeholder = if is_adding {
        "\u{8f93}\u{5165}\u{6307}\u{4ee4}\u{6570}\u{636e}..."
    } else {
        "\u{8f93}\u{5165}\u{53d1}\u{9001}\u{6570}\u{636e}..."
    };

    let input_display = if app.send_input.is_empty() && !is_focused {
        placeholder.to_string()
    } else {
        app.send_input.clone()
    };

    let cursor = if is_focused { "\u{258e}" } else { "" };

    let mut spans = Vec::new();
    spans.push(Span::raw(" "));
    spans.push(Span::styled(label, focus_style));
    spans.push(Span::styled(
        format!("{} {}", input_display, cursor),
        if is_adding {
            Style::default().fg(Color::Rgb(255, 183, 77))
        } else {
            Style::default().fg(Color::Rgb(100, 181, 246))
        },
    ));

    if !is_adding {
        let line_ending_options = ["\u{65e0}", "LF", "CR", "CRLF"];
        let current_le = match app.line_ending {
            crate::config::LineEnding::None => 0,
            crate::config::LineEnding::LF => 1,
            crate::config::LineEnding::CR => 2,
            crate::config::LineEnding::CRLF => 3,
        };

        spans.push(Span::raw("  "));
        spans.push(Span::raw("\u{884c}\u{5c3e} "));

        for (i, name) in line_ending_options.iter().enumerate() {
            let style = if i == current_le {
                Style::default().fg(Color::Rgb(255, 183, 77))
            } else {
                Style::default().fg(Color::Gray)
            };
            spans.push(Span::styled(format!("{} ", name), style));
        }

        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            "[\u{53d1}\u{9001}]",
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(21, 101, 192)),
        ));
    } else {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "[Enter保存 Esc取消]",
            Style::default().fg(Color::Yellow),
        ));
    }

    let line = Line::from(spans);
    let p = Paragraph::new(line).style(Style::default().bg(Color::Rgb(15, 15, 35)));
    f.render_widget(p, area);
}
