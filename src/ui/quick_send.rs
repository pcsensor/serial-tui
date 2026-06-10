use crate::app::CommandDialogState;
use crate::app::{App, FocusArea};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

fn fit_text(value: &str, width: u16) -> String {
    let max = width as usize;
    if max == 0 {
        return String::new();
    }
    let mut chars = value.chars();
    let mut text: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() && max > 1 {
        text.pop();
        text.push('…');
    }
    text
}

pub fn render_quick_send(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusArea::QuickSend;
    let is_adding = matches!(app.command_dialog, CommandDialogState::Open { .. });

    let hint_rows = if is_focused || is_adding { 1 } else { 0 };
    let visible = ((area.height.saturating_sub(hint_rows)) / 2).max(1) as usize;
    let start = if app.quick_send_selected >= visible {
        app.quick_send_selected - visible + 1
    } else {
        0
    };

    let mut lines = Vec::new();
    for (i, cmd) in app
        .command_list
        .commands
        .iter()
        .enumerate()
        .skip(start)
        .take(visible)
    {
        let selected = i == app.quick_send_selected && is_focused;
        let style = if i == app.quick_send_selected && is_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Rgb(255, 183, 77))
                .add_modifier(Modifier::BOLD)
        };

        let marker = if selected { "\u{25b6}" } else { " " };
        let name_width = area.width.saturating_sub(4);
        let data_width = area.width.saturating_sub(5);
        lines.push(Line::from(Span::styled(
            format!(" {} {}", marker, fit_text(&cmd.name, name_width)),
            style,
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", fit_text(&cmd.data, data_width)),
            Style::default()
                .fg(Color::Rgb(128, 151, 170))
                .add_modifier(Modifier::DIM),
        )));
    }

    // 底部提示行
    let hint = if is_adding {
        Span::styled(
            "添加中: 在弹窗填写名称和数据",
            Style::default().fg(Color::Yellow),
        )
    } else if is_focused {
        Span::styled(
            "a添加 d删除 Enter发送",
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::raw("")
    };

    if !hint.content.is_empty() {
        lines.push(Line::from(hint));
    }

    if lines.is_empty() {
        let paragraph = Paragraph::new(Line::from(Span::styled(
            "\u{6682}\u{65e0}\u{6307}\u{4ee4}",
            Style::default().fg(Color::DarkGray),
        )));
        f.render_widget(
            paragraph.style(Style::default().bg(Color::Rgb(15, 15, 35))),
            area,
        );
    } else {
        let paragraph = Paragraph::new(lines);
        f.render_widget(
            paragraph.style(Style::default().bg(Color::Rgb(15, 15, 35))),
            area,
        );
    }
}
