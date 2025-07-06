use crate::app::entry::InputEntry;
use crate::app::tui::widgets::Blink::Show;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Layout, Line, Stylize, Text, Widget};
use ratatui::widgets::Block;
use ratatui::widgets::{Clear, Paragraph, Wrap};
use std::time::Duration;

pub mod dashboard;
mod editing;
pub mod help;
mod options;

impl Widget for &InputEntry {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::bordered().border_type(ratatui::widgets::BorderType::Plain);
        block.render(area, buf);
        Clear.render(area, buf);
        let name = self.name.as_str();
        let desc = self.description.as_str();
        let identity = self.identity.as_str();
        let password = self.password.as_str();
        let rc = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Fill(0),
            ])
            .split(area);

        let b_name = Block::bordered().title("name").fg(Color::White);
        let b_ident = Block::bordered().title("identity").fg(Color::Red);
        let b_password = Block::bordered().title("password").fg(Color::Red);
        let b_description = Block::bordered().title("description").fg(Color::White);

        Paragraph::new(name).block(b_name).render(rc[0], buf);
        Paragraph::new(identity).block(b_ident).render(rc[1], buf);
        Paragraph::new(password)
            .block(b_password)
            .render(rc[2], buf);
        Paragraph::new(desc)
            .wrap(Wrap { trim: false })
            .block(b_description)
            .render(rc[3], buf);
    }
}

impl InputField {
    pub fn current_weight_paragraph(&self) -> Paragraph {
        // 渲染带光标的文本
        let cursor_char = if self.current_blink() == Show {
            '|'
        } else {
            ' '
        };
        let text = format!(
            "{}{}{}",
            self.pre_cursor_text(),
            cursor_char,
            &self.content[self.cursor_position..]
        );

        Paragraph::new(text)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Blink {
    None,
    Show,
    Hide,
}
/// 有光标的输入框
pub struct InputField {
    content: String,        // 内容
    cursor_position: usize, // 光标位置
    blink: Blink,           // 光标闪烁状态
}

impl InputField {
    pub fn new_no_blink() -> InputField {
        InputField {
            content: String::new(),
            cursor_position: 0,
            blink: Blink::None,
        }
    }
    pub fn new_blink() -> InputField {
        InputField {
            content: String::new(),
            cursor_position: 0,
            blink: Blink::Show,
        }
    }

    pub fn current_content(&self) -> &str {
        &self.content
    }

    pub fn current_blink(&self) -> Blink {
        self.blink
    }

    /// 处理字符输入
    pub fn input_char(&mut self, c: char) {
        self.content.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
    }

    /// 处理退格
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            let char_len = self.content[..self.cursor_position]
                .chars()
                .last()
                .unwrap()
                .len_utf8();
            self.cursor_position -= char_len;
            self.content.remove(self.cursor_position);
        }
    }

    // 获取光标前的文本（用于计算光标位置）
    fn pre_cursor_text(&self) -> &str {
        &self.content[..self.cursor_position]
    }

    pub fn cursor_to_left(&mut self) {
        self.move_cursor(-1);
    }
    pub fn cursor_to_right(&mut self) {
        self.move_cursor(1);
    }

    // 移动光标
    fn move_cursor(&mut self, direction: i32) {
        match direction {
            -1 => {
                // 左移
                if self.cursor_position > 0 {
                    let char_len = self.content[..self.cursor_position]
                        .chars()
                        .last()
                        .unwrap()
                        .len_utf8();
                    self.cursor_position -= char_len;
                }
            }
            1 => {
                // 右移
                if self.cursor_position < self.content.len() {
                    let char_len = self.content[self.cursor_position..]
                        .chars()
                        .next()
                        .unwrap()
                        .len_utf8();
                    self.cursor_position += char_len;
                }
            }
            _ => {}
        }
    }

    /// 更新光标闪烁状态
    /// 当光标闪烁状态为 Show 时，更新为 Hide
    /// 当光标闪烁状态为 Hide 时，更新为 Show
    /// 当光标闪烁状态为 None 时，不更新
    pub fn update_blink(&mut self) {
        match self.blink {
            Blink::Show => {
                self.blink = Blink::Hide;
            }
            Blink::Hide => {
                self.blink = Blink::Show;
            }
            _ => (),
        }
    }
}
