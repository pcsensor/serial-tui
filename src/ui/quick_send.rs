use crate::app::{App, FocusArea, QuickSendMode};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_quick_send(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == FocusArea::QuickSend;

    let visible = area.height.saturating_sub(3).max(5) as usize;
    let start = if app.quick_send_selected >= visible {
        app.quick_send_selected - visible + 1
    } else {
        0
    };

    let lines: Vec<Line> = app
        .command_list
        .commands
        .iter()
        .enumerate()
        .skip(start)
        .take(visible)
        .map(|(i, cmd)| {
            let style = if i == app.quick_send_selected && is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(255, 183, 77))
            };
            Line::from(Span::styled(
                format!(
                    " {} {}",
                    if i == app.quick_send_selected && is_focused {
                        "\u{25b6}"
                    } else {
                        " "
                    },
                    cmd.name
                ),
                style,
            ))
        })
        .collect();

    // 底部提示行
    let hint = match app.quick_send_mode {
        QuickSendMode::Adding => Span::styled(
            "输入指令数据后回车 (Esc取消)",
            Style::default().fg(Color::Yellow),
        ),
        QuickSendMode::Normal => {
            if is_focused {
                Span::styled(
                    "a添加 d删除 Enter发送",
                    Style::default().fg(Color::DarkGray),
                )
            } else {
                Span::raw("")
            }
        }
    };

    let mut all_lines = lines;
    if !hint.content.is_empty() {
        all_lines.push(Line::from(hint));
    }

    if all_lines.is_empty() {
        let paragraph = Paragraph::new(Line::from(Span::styled(
            "\u{6682}\u{65e0}\u{6307}\u{4ee4}",
            Style::default().fg(Color::DarkGray),
        )));
        f.render_widget(
            paragraph.style(Style::default().bg(Color::Rgb(15, 15, 35))),
            area,
        );
    } else {
        let paragraph = Paragraph::new(all_lines);
        f.render_widget(
            paragraph.style(Style::default().bg(Color::Rgb(15, 15, 35))),
            area,
        );
    }
}
