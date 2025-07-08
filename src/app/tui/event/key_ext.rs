use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub trait KeyEventExt {
    /// F1
    fn _is_f1(&self) -> bool;
    /// 退出
    fn _is_ctrl_c(&self) -> bool;
    /// 下-半页
    fn _is_ctrl_d(&self) -> bool;
    /// 上-半页
    fn _is_ctrl_u(&self) -> bool;
    /// 保存
    fn _is_ctrl_s(&self) -> bool;
    /// q-无论大小写
    fn _is_q_ignore_case(&self) -> bool;
    /// i-无论大小写
    fn _is_i_ignore_case(&self) -> bool;
    /// u-无论大小写
    fn _is_u_ignore_case(&self) -> bool;
    /// o-无论大小写
    fn _is_o_ignore_case(&self) -> bool;
    /// tab
    fn _is_tab(&self) -> bool;
    /// 上-键盘上
    fn _is_up(&self) -> bool;
    /// 下-键盘下
    fn _is_down(&self) -> bool;
    /// j
    fn _is_j(&self) -> bool;
    /// k
    fn _is_k(&self) -> bool;
    /// d
    fn _is_d(&self) -> bool;
    /// enter
    fn _is_enter(&self) -> bool;
    /// backspace
    fn _is_backspace(&self) -> bool;
    /// ESC
    fn _is_esc(&self) -> bool;
}

/// 判定是否按下 ctrl 和 某个字母键
pub fn _is_press_ctrl_and_char(key_event: &KeyEvent, char: char) -> bool {
    if key_event.modifiers == KeyModifiers::CONTROL {
        key_event.code == KeyCode::Char(char)
    } else {
        false
    }
}

/// 判定是否按下 某个字母键-忽略大小写
pub fn _is_press_char_ignore_case(key_event: &KeyEvent, char: char) -> bool {
    key_event.code == KeyCode::Char(char.to_ascii_lowercase())
        || key_event.code == KeyCode::Char(char.to_ascii_uppercase())
}

impl KeyEventExt for KeyEvent {
    fn _is_f1(&self) -> bool {
        self.code == KeyCode::F(1)
    }

    fn _is_ctrl_c(&self) -> bool {
        if let KeyCode::Char('c' | 'C') = self.code {
            self.modifiers == KeyModifiers::CONTROL
        } else {
            false
        }
    }

    fn _is_ctrl_d(&self) -> bool {
        _is_press_ctrl_and_char(self, 'd')
    }

    fn _is_ctrl_u(&self) -> bool {
        _is_press_ctrl_and_char(self, 'u')
    }

    fn _is_ctrl_s(&self) -> bool {
        _is_press_ctrl_and_char(self, 's')
    }

    fn _is_q_ignore_case(&self) -> bool {
        _is_press_char_ignore_case(self, 'q')
    }

    fn _is_i_ignore_case(&self) -> bool {
        _is_press_char_ignore_case(self, 'i')
    }

    fn _is_u_ignore_case(&self) -> bool {
        _is_press_char_ignore_case(self, 'u')
    }

    fn _is_o_ignore_case(&self) -> bool {
        _is_press_char_ignore_case(self, 'o')
    }

    fn _is_tab(&self) -> bool {
        self.code == KeyCode::Tab
    }

    fn _is_up(&self) -> bool {
        self.code == KeyCode::Up
    }

    fn _is_down(&self) -> bool {
        self.code == KeyCode::Down
    }

    fn _is_j(&self) -> bool {
        self.code == KeyCode::Char('j')
    }

    fn _is_k(&self) -> bool {
        self.code == KeyCode::Char('k')
    }

    fn _is_d(&self) -> bool {
        self.code == KeyCode::Char('d')
    }

    fn _is_enter(&self) -> bool {
        self.code == KeyCode::Enter
    }

    fn _is_backspace(&self) -> bool {
        self.code == KeyCode::Backspace
    }

    fn _is_esc(&self) -> bool {
        self.code == KeyCode::Esc
    }
}
