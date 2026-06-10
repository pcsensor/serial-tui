# Tab 切换指示器 + 导出对话框 设计规格

## 概述

本规格涵盖两个独立但同批实现的 UI 增强：
1. **Tab 切换面板指示器**：用右上角面包屑浮层替代底部状态栏的文字提示
2. **Ctrl+E 导出对话框**：增加格式（CSV/TXT）和目录选择，替代当前的直接导出

## 1. Tab 切换指示器

### 用户体验

按 `Tab` 键切换面板时，在终端数据区的右上角弹出一个小浮层，显示全部 4 个面板名称，当前激活面板以黄底黑字高亮。浮层约 **2 秒**后自动消失。

### 视觉规格

```
┌ TAB ▸ ──────────┐
│  设置栏          │
│  发送栏          │
│ ▶收发区◀         │  ← 当前激活：bg(Yellow) fg(Black)
│  快捷发送        │
└─────────────────┘
```

- 位置：terminal 区域（`middle_row[0]`）右上角，通过 `Rect { x: area.x + area.width - 20, y: area.y, width: 20, height: 6 }` 定位
- 边框：`Borders::ALL`，颜色 `Color::Rgb(255, 215, 0)`（金黄色）
- 标题：`" TAB ▸ "`，颜色 `Color::Cyan`
- 当前面板行：`Style::default().bg(Color::Yellow).fg(Color::Black)`
- 其他面板行：`Style::default().fg(Color::DarkGray)`
- 持续时间：`tab_hint_ticks = 30`，每次 `Event::Tick` 减 1，减到 0 时隐藏

### 状态变更

`App` 新增字段：
```rust
pub tab_hint_ticks: u8,
```

Tab 键处理（`src/app.rs`）：
- 切换 `focus` 后设置 `tab_hint_ticks = 30`
- **移除** `self.status_message = format!("已切换到: {}", focus_name)` 这行

Tick 处理：
- `tab_hint_ticks` 大于 0 时每 tick 减 1

### 渲染

在 `src/ui/overlay.rs`（新文件）中实现 `render_tab_hint(f, app, terminal_area)`：
- 若 `app.tab_hint_ticks == 0` 则直接返回
- 计算右上角 Rect（宽 20，高 6）
- 先 `f.render_widget(Clear, rect)` 清除背景
- 再渲染带边框的 Paragraph，4 行面板名

在 `src/ui/mod.rs` 的 `render` 函数末尾调用此函数。

---

## 2. Ctrl+E 导出对话框

### 用户体验

按 `Ctrl+E` 弹出居中模态框。用户选择格式（CSV 或 TXT）、输入保存目录，按 `Enter` 导出，按 `Esc` 取消。

### 视觉规格

```
╔═ ◈ EXPORT DATA ══════════════════════╗
║  FORMAT   [CSV]   TXT                ║
║  SAVE TO  ~/Desktop/▌                ║
╟──────────────────────────────────────╢
║  [Esc] 取消            [Enter] 导出  ║
╚══════════════════════════════════════╝
```

- 尺寸：宽 44，高 7
- 位置：整个终端（`f.area()`）居中
- 边框：`Borders::ALL`，颜色 `Color::Rgb(0, 212, 255)`（青色）
- 标题：`" ◈ EXPORT DATA "`
- 激活格式：`Style::default().bg(Color::Yellow).fg(Color::Black)`
- 非激活格式：`Style::default().fg(Color::DarkGray)`
- 目录行聚焦时：末尾显示光标 `▌`，边框颜色改为 `Color::Yellow`
- 底部按键提示：固定文字，灰色

### 新增类型（`src/app.rs`）

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

`App` 新增字段：
```rust
pub export_dialog: ExportDialogState,
```

初始值：`ExportDialogState::Hidden`

### 键盘交互（对话框打开时优先处理）

在 `handle_key` 顶部，若 `export_dialog != Hidden` 则拦截所有按键：

| 按键 | 字段=Format | 字段=Dir |
|------|------------|---------|
| `Tab` / `←` `→` | 切换 CSV↔TXT | 无效 |
| `↓` / `↑` | 焦点移到 Dir | 焦点移到 Format |
| 普通字符 | 无效 | 追加到 dir |
| `Backspace` | 无效 | dir 删最后一字符 |
| `Enter` | 执行导出并关闭 | 执行导出并关闭 |
| `Esc` | 关闭对话框 | 关闭对话框 |

`Ctrl+E` 触发：
- 若数据为空：`status_message = "无数据可导出"`，不打开对话框
- 否则：`export_dialog = ExportDialogState::Open { format: Csv, dir: "./".to_string(), field: Format }`

### 导出执行逻辑

```rust
fn do_export(&mut self, format: ExportFormat, dir: String) {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let ext = match format { ExportFormat::Csv => "csv", ExportFormat::Txt => "txt" };
    let filename = format!("{}/serial_export_{}.{}", dir.trim_end_matches('/'), timestamp, ext);
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

### 渲染

在 `src/ui/overlay.rs` 中实现 `render_export_dialog(f, app)`：
- 若 `export_dialog == Hidden` 则返回
- 计算居中 Rect（宽 44，高 7）
- `f.render_widget(Clear, rect)`
- 渲染 Block（边框 + 标题）
- 渲染 3 行内容：格式选择行、目录输入行、按键提示行

---

## 涉及文件汇总

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/app.rs` | 修改 | 新增 `tab_hint_ticks`、`export_dialog` 字段及相关枚举；更新 Tab/Ctrl+E/Tick 处理逻辑 |
| `src/ui/overlay.rs` | 新建 | `render_tab_hint` + `render_export_dialog` 两个渲染函数 |
| `src/ui/mod.rs` | 修改 | 引入 overlay 模块，在 `render` 末尾调用两个浮层渲染 |
| `src/ui/status.rs` | 修改 | 移除焦点文本显示（`focus_text` 相关 Span） |
| `src/export.rs` | 无需修改 | `export_text` 函数已存在 |

## 验收标准

- [ ] 按 Tab 后右上角出现面包屑浮层，当前面板黄色高亮，2 秒后消失
- [ ] 底部状态栏不再显示"焦点: XXX"文字
- [ ] Ctrl+E 弹出对话框，可选 CSV/TXT
- [ ] 对话框可输入目录路径
- [ ] Enter 按当前格式和目录导出，状态栏显示文件路径
- [ ] Esc 取消，不产生任何文件
- [ ] 无数据时 Ctrl+E 不打开对话框，直接提示
