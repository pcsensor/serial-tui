use crate::app::{
    App, CommandDialogState, CommandField, ExportDialogState, ExportField, ExportFormat, FocusArea,
};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_tab_hint(f: &mut Frame, app: &App, terminal_area: Rect) {
    if app.tab_hint_ticks == 0 {
        return;
    }

    let w = 18u16;
    let h = 6u16;
    if terminal_area.width < w || terminal_area.height < h {
        return;
    }

    let x = terminal_area.x + terminal_area.width - w;
    let y = terminal_area.y;
    let rect = Rect::new(x, y, w, h);

    f.render_widget(Clear, rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" TAB ▸ ", Style::default().fg(Color::Cyan)))
        .border_style(Style::default().fg(Color::Rgb(255, 215, 0)));

    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let panels: [(FocusArea, &str); 4] = [
        (FocusArea::Settings, " 设置栏"),
        (FocusArea::Terminal, " 收发区"),
        (FocusArea::SendInput, " 发送栏"),
        (FocusArea::QuickSend, " 快捷发送"),
    ];

    let lines: Vec<Line> = panels
        .iter()
        .map(|(area, name)| {
            if *area == app.focus {
                Line::from(Span::styled(
                    format!("▶{}", name),
                    Style::default().bg(Color::Yellow).fg(Color::Black),
                ))
            } else {
                Line::from(Span::styled(
                    format!(" {}", name),
                    Style::default().fg(Color::DarkGray),
                ))
            }
        })
        .collect();

    f.render_widget(Paragraph::new(lines), inner);
}

pub fn render_export_dialog(f: &mut Frame, app: &App) {
    let (format, dir, field) = match &app.export_dialog {
        ExportDialogState::Hidden => return,
        ExportDialogState::Open { format, dir, field } => (format, dir, field),
    };

    let area = f.area();
    let w = 54u16;
    let h = 10u16;
    if area.width < w || area.height < h {
        return;
    }
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let rect = Rect::new(x, y, w.min(area.width), h.min(area.height));

    f.render_widget(Clear, rect);

    let border_color = Color::Rgb(0, 212, 255);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " 导出数据 ",
            Style::default()
                .fg(Color::Rgb(0, 212, 255))
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let csv_style = if *format == ExportFormat::Csv {
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(116, 136, 153))
    };
    let txt_style = if *format == ExportFormat::Txt {
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(116, 136, 153))
    };

    let title_line = Line::from(Span::styled(
        "  选择导出格式和保存目录",
        Style::default().fg(Color::Rgb(128, 151, 170)),
    ));
    let format_line = Line::from(vec![
        Span::styled(
            if *field == ExportField::Format {
                "▶ 格式  "
            } else {
                "  格式  "
            },
            Style::default().fg(if *field == ExportField::Format {
                Color::Yellow
            } else {
                Color::Gray
            }),
        ),
        Span::styled(" CSV ", csv_style),
        Span::raw("  "),
        Span::styled(" TXT ", txt_style),
    ]);

    let cursor = if *field == ExportField::Dir {
        "▌"
    } else {
        ""
    };
    let dir_color = if *field == ExportField::Dir {
        Color::Rgb(0, 212, 255)
    } else {
        Color::Gray
    };
    let dir_line = Line::from(vec![
        Span::styled(
            if *field == ExportField::Dir {
                "▶ 目录  "
            } else {
                "  目录  "
            },
            Style::default().fg(if *field == ExportField::Dir {
                Color::Yellow
            } else {
                Color::Gray
            }),
        ),
        Span::styled(format!("{}{}", dir, cursor), Style::default().fg(dir_color)),
    ]);

    let hint_line = Line::from(vec![
        Span::styled("  Tab/↑↓ 切换", Style::default().fg(Color::DarkGray)),
        Span::raw("        "),
        Span::styled("Esc 取消", Style::default().fg(Color::DarkGray)),
        Span::raw("        "),
        Span::styled("Enter 导出", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(vec![
        title_line,
        Line::raw(""),
        format_line,
        Line::raw(""),
        dir_line,
        Line::raw(""),
        hint_line,
    ])
    .wrap(Wrap { trim: true });
    f.render_widget(paragraph, inner);
}

pub fn render_command_dialog(f: &mut Frame, app: &App) {
    let (name, data, field) = match &app.command_dialog {
        CommandDialogState::Hidden => return,
        CommandDialogState::Open { name, data, field } => (name, data, field),
    };

    let area = f.area();
    let w = 54u16;
    let h = 11u16;
    if area.width < w || area.height < h {
        return;
    }
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let rect = Rect::new(x, y, w.min(area.width), h.min(area.height));

    f.render_widget(Clear, rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " 添加快捷指令 ",
            Style::default()
                .fg(Color::Rgb(255, 183, 77))
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Rgb(255, 183, 77)));

    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let name_cursor = if *field == CommandField::Name {
        "▌"
    } else {
        ""
    };
    let data_cursor = if *field == CommandField::Data {
        "▌"
    } else {
        ""
    };
    let name_value = if name.is_empty() && *field != CommandField::Name {
        "未命名".to_string()
    } else {
        format!("{}{}", name, name_cursor)
    };
    let data_value = if data.is_empty() && *field != CommandField::Data {
        "输入要发送的数据".to_string()
    } else {
        format!("{}{}", data, data_cursor)
    };

    let title_line = Line::from(Span::styled(
        "  为常用发送内容保存一个名称和数据",
        Style::default().fg(Color::Rgb(128, 151, 170)),
    ));
    let name_line = Line::from(vec![
        Span::styled(
            if *field == CommandField::Name {
                "▶ 名称  "
            } else {
                "  名称  "
            },
            Style::default().fg(if *field == CommandField::Name {
                Color::Yellow
            } else {
                Color::Gray
            }),
        ),
        Span::styled(
            name_value,
            Style::default().fg(if *field == CommandField::Name {
                Color::Rgb(255, 183, 77)
            } else {
                Color::Rgb(176, 190, 197)
            }),
        ),
    ]);
    let data_line = Line::from(vec![
        Span::styled(
            if *field == CommandField::Data {
                "▶ 数据  "
            } else {
                "  数据  "
            },
            Style::default().fg(if *field == CommandField::Data {
                Color::Yellow
            } else {
                Color::Gray
            }),
        ),
        Span::styled(
            data_value,
            Style::default().fg(if *field == CommandField::Data {
                Color::Rgb(0, 212, 255)
            } else {
                Color::Rgb(176, 190, 197)
            }),
        ),
    ]);
    let hint_line = Line::from(vec![
        Span::styled("  Tab/↑↓ 切换", Style::default().fg(Color::DarkGray)),
        Span::raw("        "),
        Span::styled("Esc 取消", Style::default().fg(Color::DarkGray)),
        Span::raw("        "),
        Span::styled("Enter 保存", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(vec![
        title_line,
        Line::raw(""),
        name_line,
        Line::raw(""),
        data_line,
        Line::raw(""),
        hint_line,
    ])
    .wrap(Wrap { trim: true });
    f.render_widget(paragraph, inner);
}
