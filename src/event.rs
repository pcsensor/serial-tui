use crossterm::event::KeyEvent;

/// 应用事件枚举，涵盖按键、串口数据、系统事件
#[derive(Debug, Clone)]
pub enum Event {
    /// 键盘事件
    Key(KeyEvent),
    /// 串口接收数据
    RxData(Vec<u8>),
    /// 可用端口列表变化（热插拔）
    PortsChanged,
    /// 串口错误或被动断开
    SerialError(String),
    /// 定时 tick（UI 刷新、自动重连检查）
    Tick,
    /// 退出应用
    #[allow(dead_code)]
    Quit,
}
