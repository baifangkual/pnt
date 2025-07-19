use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub trait KeyEventExt {
    /// 判定是否为某char按下
    ///
    /// 注意，该方法仅判定当前code是否为指定char，不判定 modifiers 是否有 shift 按下
    fn is_char(&self, char: char) -> bool;
    /// 判定是否为按下 ctrl 同时 按下某键
    fn is_ctrl_char(&self, char: char) -> bool;
    /// F1
    fn is_f1(&self) -> bool;
    /// tab
    fn is_tab(&self) -> bool;
    /// 上-键盘上
    fn is_up(&self) -> bool;
    /// 下-键盘下
    fn is_down(&self) -> bool;
    /// enter
    fn is_enter(&self) -> bool;
    /// ESC
    fn is_esc(&self) -> bool;
}

impl KeyEventExt for KeyEvent {
    #[inline]
    fn is_char(&self, char: char) -> bool {
        self.code == KeyCode::Char(char)
    }
    #[inline]
    fn is_ctrl_char(&self, char: char) -> bool {
        self.modifiers == KeyModifiers::CONTROL && self.code == KeyCode::Char(char)
    }
    #[inline]
    fn is_f1(&self) -> bool {
        self.code == KeyCode::F(1)
    }
    #[inline]
    fn is_tab(&self) -> bool {
        self.code == KeyCode::Tab
    }
    #[inline]
    fn is_up(&self) -> bool {
        self.code == KeyCode::Up
    }
    #[inline]
    fn is_down(&self) -> bool {
        self.code == KeyCode::Down
    }
    #[inline]
    fn is_enter(&self) -> bool {
        self.code == KeyCode::Enter
    }
    #[inline]
    fn is_esc(&self) -> bool {
        self.code == KeyCode::Esc
    }
}
