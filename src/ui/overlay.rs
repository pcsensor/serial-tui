use crate::app::{App, ExportDialogState, ExportField, ExportFormat, FocusArea};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
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
    let w = 44u16;
    let h = 7u16;
    if area.width < w || area.height < h {
        return;
    }
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let rect = Rect::new(x, y, w.min(area.width), h.min(area.height));

    f.render_widget(Clear, rect);

    let border_color = if *field == ExportField::Dir {
        Color::Yellow
    } else {
        Color::Rgb(0, 212, 255)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " ◈ EXPORT DATA ",
            Style::default().fg(Color::Rgb(0, 212, 255)),
        ))
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let csv_style = if *format == ExportFormat::Csv {
        Style::default().bg(Color::Yellow).fg(Color::Black)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let txt_style = if *format == ExportFormat::Txt {
        Style::default().bg(Color::Yellow).fg(Color::Black)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let format_line = Line::from(vec![
        Span::styled("  FORMAT  ", Style::default().fg(Color::Gray)),
        Span::styled(" CSV ", csv_style),
        Span::raw("  "),
        Span::styled(" TXT ", txt_style),
    ]);

    let cursor = if *field == ExportField::Dir { "▌" } else { "" };
    let dir_color = if *field == ExportField::Dir {
        Color::Rgb(0, 212, 255)
    } else {
        Color::Gray
    };
    let dir_line = Line::from(vec![
        Span::styled("  SAVE TO  ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}{}", dir, cursor),
            Style::default().fg(dir_color),
        ),
    ]);

    let hint_line = Line::from(vec![
        Span::styled("  [Esc]取消", Style::default().fg(Color::DarkGray)),
        Span::raw("              "),
        Span::styled("[Enter]导出  ", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(vec![
        format_line,
        dir_line,
        Line::raw(""),
        hint_line,
    ]);
    f.render_widget(paragraph, inner);
}
