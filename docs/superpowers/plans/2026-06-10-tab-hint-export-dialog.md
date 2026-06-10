# Tab 切换指示器 + 导出对话框 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 用右上角面包屑浮层替换 Tab 切换的底部状态栏提示；Ctrl+E 弹出居中模态对话框，支持选择 CSV/TXT 格式和输入目录路径后导出。

**架构：** `src/app.rs` 新增两个状态字段（`tab_hint_ticks: u8`、`export_dialog: ExportDialogState`）和三个枚举类型；新建 `src/ui/overlay.rs` 集中实现两个浮层的渲染；`src/ui/mod.rs` 在所有面板渲染之后调用浮层，使其叠加在最顶层；`src/ui/status.rs` 移除原焦点提示文字。

**技术栈：** Rust、ratatui 0.28、crossterm 0.28、tokio

---

## 文件清单

| 文件 | 操作 | 职责 |
|------|------|------|
| `src/app.rs` | 修改 | 新增枚举类型和状态字段；更新 Tab/Ctrl+E/Tick 键处理逻辑 |
| `src/ui/overlay.rs` | 新建 | `render_tab_hint` + `render_export_dialog` 两个渲染函数 |
| `src/ui/mod.rs` | 修改 | 引入 overlay 模块，在 `render` 末尾调用两个浮层 |
| `src/ui/status.rs` | 修改 | 移除焦点文本（`focus_text` 相关 Span） |

`src/export.rs` 无需修改——`export_csv` 和 `export_text` 函数已存在。

---

## 任务 1：App 新增状态枚举和字段

**文件：**
- 修改：`src/app.rs`

- [ ] **步骤 1：在 `src/app.rs` 末尾（`#[cfg(test)]` 之前）写失败测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_tab_hint_ticks_is_zero() {
        let app = App::new();
        assert_eq!(app.tab_hint_ticks, 0);
    }

    #[test]
    fn test_initial_export_dialog_is_hidden() {
        let app = App::new();
        assert!(matches!(app.export_dialog, ExportDialogState::Hidden));
    }
}
```

- [ ] **步骤 2：运行测试确认编译失败**

```bash
cargo test test_initial 2>&1 | head -20
```

预期：编译错误，`tab_hint_ticks` 和 `ExportDialogState` 未定义。

- [ ] **步骤 3：在 `src/app.rs` 顶部（`pub enum FocusArea` 之后）添加三个枚举**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Txt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportField {
    Format,
    Dir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportDialogState {
    Hidden,
    Open {
        format: ExportFormat,
        dir: String,
        field: ExportField,
    },
}
```

- [ ] **步骤 4：在 `App` 结构体中添加两个字段**

在 `pub rx_bytes: u64,` 之后添加：

```rust
pub tab_hint_ticks: u8,
pub export_dialog: ExportDialogState,
```

- [ ] **步骤 5：在 `App::new()` 中初始化新字段**

在 `rx_bytes: 0,` 之后添加：

```rust
tab_hint_ticks: 0,
export_dialog: ExportDialogState::Hidden,
```

- [ ] **步骤 6：运行测试确认通过**

```bash
cargo test test_initial
```

预期：2 个测试 PASS。

- [ ] **步骤 7：Commit**

```bash
git add src/app.rs
git commit -m "feat: 新增 tab_hint_ticks 与 ExportDialogState 状态字段"
```

---

## 任务 2：Tab 键处理 + Tick 递减

**文件：**
- 修改：`src/app.rs`

- [ ] **步骤 1：在测试模块添加失败测试**

在 `#[cfg(test)] mod tests` 中追加：

```rust
#[test]
fn test_tab_sets_hint_ticks() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = App::new();
    app.tab_hint_ticks = 0;
    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
    assert_eq!(app.tab_hint_ticks, 20);
}

#[test]
fn test_tick_decrements_hint_ticks() {
    let mut app = App::new();
    app.tab_hint_ticks = 5;
    app.handle_event(Event::Tick);
    assert_eq!(app.tab_hint_ticks, 4);
}

#[test]
fn test_tick_does_not_underflow() {
    let mut app = App::new();
    app.tab_hint_ticks = 0;
    app.handle_event(Event::Tick);
    assert_eq!(app.tab_hint_ticks, 0);
}
```

- [ ] **步骤 2：运行测试确认失败**

```bash
cargo test test_tab_sets_hint_ticks test_tick_decrements test_tick_does_not
```

预期：3 个测试 FAIL（ticks 不变）。

- [ ] **步骤 3：修改 Tab 键处理（`handle_key` 中的 `KeyCode::Tab` 分支）**

找到：
```rust
KeyEvent {
    code: KeyCode::Tab, ..
} => {
    self.focus = match self.focus { ... };
    let focus_name = match self.focus { ... };
    self.status_message = format!("已切换到: {}", focus_name);
    return;
}
```

替换为（删除 `focus_name` 变量和 `status_message` 赋值，添加 ticks）：
```rust
KeyEvent {
    code: KeyCode::Tab, ..
} => {
    self.focus = match self.focus {
        FocusArea::Settings => FocusArea::Terminal,
        FocusArea::Terminal => FocusArea::SendInput,
        FocusArea::SendInput => FocusArea::QuickSend,
        FocusArea::QuickSend => FocusArea::Settings,
    };
    self.tab_hint_ticks = 20;
    return;
}
```

- [ ] **步骤 4：在 `Event::Tick` 分支中添加 ticks 递减**

找到：
```rust
Event::Tick => {
    if self.serial_manager.should_reconnect() {
        ...
    }
}
```

在 `if self.serial_manager.should_reconnect()` **之前**添加：
```rust
if self.tab_hint_ticks > 0 {
    self.tab_hint_ticks -= 1;
}
```

- [ ] **步骤 5：运行测试确认通过**

```bash
cargo test test_tab_sets_hint_ticks test_tick_decrements test_tick_does_not
```

预期：3 个测试 PASS。

- [ ] **步骤 6：Commit**

```bash
git add src/app.rs
git commit -m "feat: Tab 键设置 tab_hint_ticks，Tick 递减"
```

---

## 任务 3：导出对话框键处理

**文件：**
- 修改：`src/app.rs`

- [ ] **步骤 1：在测试模块添加失败测试**

```rust
#[test]
fn test_ctrl_e_no_data_no_dialog() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = App::new();
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('e'), KeyModifiers::CONTROL,
    )));
    assert!(matches!(app.export_dialog, ExportDialogState::Hidden));
    assert!(!app.status_message.is_empty());
}

#[test]
fn test_ctrl_e_with_data_opens_dialog() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = App::new();
    app.data_records.push(crate::export::DataRecord {
        time: "12:00:00.000".into(),
        dir: "RX".into(),
        data: "test".into(),
    });
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('e'), KeyModifiers::CONTROL,
    )));
    assert!(matches!(app.export_dialog, ExportDialogState::Open { .. }));
}

#[test]
fn test_export_dialog_tab_switches_format() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = App::new();
    app.export_dialog = ExportDialogState::Open {
        format: ExportFormat::Csv,
        dir: "./".to_string(),
        field: ExportField::Format,
    };
    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
    assert!(matches!(
        app.export_dialog,
        ExportDialogState::Open { format: ExportFormat::Txt, .. }
    ));
}

#[test]
fn test_export_dialog_esc_closes() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = App::new();
    app.export_dialog = ExportDialogState::Open {
        format: ExportFormat::Csv,
        dir: "./".to_string(),
        field: ExportField::Format,
    };
    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
    assert!(matches!(app.export_dialog, ExportDialogState::Hidden));
}

#[test]
fn test_export_dialog_dir_input() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = App::new();
    app.export_dialog = ExportDialogState::Open {
        format: ExportFormat::Csv,
        dir: "~/".to_string(),
        field: ExportField::Dir,
    };
    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)));
    assert!(matches!(
        &app.export_dialog,
        ExportDialogState::Open { dir, .. } if dir == "~/x"
    ));
}
```

- [ ] **步骤 2：运行测试确认失败**

```bash
cargo test test_ctrl_e test_export_dialog 2>&1 | grep -E "FAILED|error"
```

预期：编译通过，但逻辑测试 FAIL（Ctrl+E 直接导出而不打开对话框）。

- [ ] **步骤 3：重构 `handle_key` 开头——提取 Ctrl+Q 并插入对话框拦截**

`handle_key` 目前是一个大 `match key { ... }` 块，无法在块中间插入 `if` 语句。需要将 Ctrl+Q 提取到独立 `if` 检查，再添加对话框拦截。

找到 `fn handle_key` 的整个开头（第一个 `match key {` 直到其对应的 `}`），将最开始的结构替换为：

```rust
fn handle_key(&mut self, key: KeyEvent) {
    // Ctrl+Q 始终退出，即使对话框打开时也有效
    if matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    ) {
        self.save_config();
        self.command_list.save().ok();
        self.running = false;
        return;
    }

    // 导出对话框打开时拦截所有其他按键
    if let ExportDialogState::Open { .. } = &self.export_dialog {
        self.handle_export_dialog_key(key);
        return;
    }

    match key {
        // 原来的 Ctrl+Q 分支已移到上方，此处删除
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            // ... 保持不变
        }
        // ... 其余分支保持不变，Ctrl+E 分支改为调用 open_export_dialog ...
        KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            self.open_export_dialog();
            return;
        }
        // ... Tab 分支等保持不变 ...
        _ => {}
    }

    match self.focus {
        // ... 保持不变
    }
}
```

具体操作：
1. 在 `fn handle_key` 的第一行 `match key {` **之前**插入 Ctrl+Q 的 `if matches!` 检查和对话框拦截代码
2. 在 `match key { ... }` 内**删除** Ctrl+Q 的 `KeyEvent { code: KeyCode::Char('q'), ... }` arm
3. 在 `match key { ... }` 内将 Ctrl+E arm 的 `self.export_data()` 改为 `self.open_export_dialog()`

- [ ] **步骤 4：（无单独步骤，步骤 3 已包含所有 handle_key 修改）**

- [ ] **步骤 5：添加 `open_export_dialog` 方法**

在 `fn export_data` 之后添加：

```rust
fn open_export_dialog(&mut self) {
    if self.data_records.is_empty() {
        self.status_message = "无数据可导出".to_string();
        return;
    }
    self.export_dialog = ExportDialogState::Open {
        format: ExportFormat::Csv,
        dir: "./".to_string(),
        field: ExportField::Format,
    };
}
```

- [ ] **步骤 6：添加 `handle_export_dialog_key` 方法**

```rust
fn handle_export_dialog_key(&mut self, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            self.export_dialog = ExportDialogState::Hidden;
        }
        KeyCode::Enter => {
            let (format, dir) = if let ExportDialogState::Open { format, dir, .. } = &self.export_dialog {
                (*format, dir.clone())
            } else {
                return;
            };
            self.export_dialog = ExportDialogState::Hidden;
            self.do_export(format, dir);
        }
        KeyCode::Tab => {
            if let ExportDialogState::Open { format, .. } = &mut self.export_dialog {
                *format = match format {
                    ExportFormat::Csv => ExportFormat::Txt,
                    ExportFormat::Txt => ExportFormat::Csv,
                };
            }
        }
        KeyCode::Down => {
            if let ExportDialogState::Open { field, .. } = &mut self.export_dialog {
                *field = ExportField::Dir;
            }
        }
        KeyCode::Up => {
            if let ExportDialogState::Open { field, .. } = &mut self.export_dialog {
                *field = ExportField::Format;
            }
        }
        KeyCode::Char(c) => {
            if let ExportDialogState::Open { field, dir, .. } = &mut self.export_dialog {
                if *field == ExportField::Dir {
                    dir.push(c);
                }
            }
        }
        KeyCode::Backspace => {
            if let ExportDialogState::Open { field, dir, .. } = &mut self.export_dialog {
                if *field == ExportField::Dir {
                    dir.pop();
                }
            }
        }
        _ => {}
    }
}
```

- [ ] **步骤 7：添加 `do_export` 方法**

```rust
fn do_export(&mut self, format: ExportFormat, dir: String) {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let ext = match format {
        ExportFormat::Csv => "csv",
        ExportFormat::Txt => "txt",
    };
    let dir = dir.trim_end_matches('/').to_string();
    let filename = format!("{}/serial_export_{}.{}", dir, timestamp, ext);
    let result = match format {
        ExportFormat::Csv => crate::export::export_csv(&self.data_records, &filename),
        ExportFormat::Txt => crate::export::export_text(&self.data_records, &filename),
    };
    self.status_message = match result {
        Ok(_) => format!("已导出: {}", filename),
        Err(e) => format!("导出失败: {}", e),
    };
}
```

- [ ] **步骤 8：运行所有测试**

```bash
cargo test
```

预期：所有测试 PASS（包括任务 1、2 中的测试）。

- [ ] **步骤 9：Commit**

```bash
git add src/app.rs
git commit -m "feat: 导出对话框状态机与键盘交互逻辑"
```

---

## 任务 4：新建 `src/ui/overlay.rs`

**文件：**
- 新建：`src/ui/overlay.rs`

- [ ] **步骤 1：创建文件，添加完整实现**

```rust
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
```

- [ ] **步骤 2：编译检查**

```bash
cargo check 2>&1 | head -30
```

预期：`overlay.rs` 模块本身无错误（mod 还未引入，会有 dead_code 警告是正常的）。

- [ ] **步骤 3：Commit**

```bash
git add src/ui/overlay.rs
git commit -m "feat: 新增 overlay 模块，实现 Tab 浮层与导出对话框渲染"
```

---

## 任务 5：接入 `mod.rs` + 清理 `status.rs`

**文件：**
- 修改：`src/ui/mod.rs`
- 修改：`src/ui/status.rs`

- [ ] **步骤 1：在 `src/ui/mod.rs` 中引入 overlay 模块并调用**

在文件顶部模块声明区域添加：
```rust
pub mod overlay;
```

在 `render` 函数末尾（`status::render_status_bar(...)` 之后）追加：
```rust
overlay::render_tab_hint(f, app, middle_row[0]);
overlay::render_export_dialog(f, app);
```

- [ ] **步骤 2：在 `src/ui/status.rs` 中移除焦点文本**

找到并删除以下代码（约第 28-33 行）：
```rust
let focus_text = match app.focus {
    FocusArea::Settings => "焦点: 设置栏",
    FocusArea::Terminal => "焦点: 收发区",
    FocusArea::SendInput => "焦点: 发送栏",
    FocusArea::QuickSend => "焦点: 快捷发送",
};
```

同时删除 `line` 的 `vec!` 中对应的两个 Span：
```rust
Span::raw(format!(" | {}", focus_text)),
```

同时删除顶部不再使用的 `use crate::app::{App, FocusArea};` 中的 `FocusArea`，改为：
```rust
use crate::app::App;
```

- [ ] **步骤 3：编译检查**

```bash
cargo check
```

预期：编译成功，无错误。

- [ ] **步骤 4：运行全部测试**

```bash
cargo test
```

预期：所有测试 PASS。

- [ ] **步骤 5：Commit**

```bash
git add src/ui/mod.rs src/ui/status.rs
git commit -m "feat: 接入 overlay 渲染，移除状态栏焦点文本"
```

---

## 任务 6：编译验证 + 人工测试

**文件：** 无代码修改

- [ ] **步骤 1：Release 编译**

```bash
cargo build --release 2>&1 | tail -5
```

预期：`Finished release [optimized]` 无警告。

- [ ] **步骤 2：运行并验证 Tab 指示器**

```bash
cargo run
```

操作：按 `Tab` 键 4 次循环切换，观察右上角出现面包屑浮层（金黄边框，当前面板高亮），约 2 秒后消失。

验收标准：
- 浮层正确显示所有 4 个面板名
- 当前面板黄底黑字高亮
- 底部状态栏不再出现"焦点: XXX"文字
- 浮层自动消失，不影响后续操作

- [ ] **步骤 3：验证 Ctrl+E 导出对话框**

操作：
1. 未连接时按 `Ctrl+E` → 状态栏应显示"无数据可导出"，无弹窗
2. 连接并收到数据后按 `Ctrl+E` → 居中弹出对话框
3. 按 `Tab` → CSV/TXT 切换高亮
4. 按 `↓` → 焦点移到目录框（边框变黄）
5. 输入路径（如 `~/Desktop`），`Backspace` 删除
6. 按 `Enter` → 对话框关闭，状态栏显示"已导出: ~/Desktop/serial_export_…"
7. 重开对话框，按 `Esc` → 关闭，无文件生成

- [ ] **步骤 4：Final commit（如有未提交内容）**

```bash
git status
```

若有未提交内容则补充 commit；若全部已提交则完成。

---

## 验收检查清单

- [ ] 按 Tab 后右上角出现面包屑浮层，当前面板黄色高亮，~2 秒后消失
- [ ] 底部状态栏不再显示"焦点: XXX"文字
- [ ] Ctrl+E 弹出居中对话框，可选 CSV/TXT
- [ ] 对话框可输入目录路径
- [ ] Enter 按当前格式和目录导出，状态栏显示文件完整路径
- [ ] Esc 取消，不产生任何文件
- [ ] 无数据时 Ctrl+E 不打开对话框，直接提示"无数据可导出"
- [ ] `cargo test` 全部通过
- [ ] `cargo build --release` 无错误
