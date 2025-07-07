use crate::app::consts::ALLOC_VALID_MAIN_PASS_MAX;
use crate::app::entry::InputEntry;
use crate::app::tui::screen::states::NeedMainPwdState;
use crate::app::tui::widgets::Blink::Show;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Rect};
use ratatui::prelude::{Color, Layout, Line, Stylize, Widget};
use ratatui::widgets::{Block, Borders, Padding};
use ratatui::widgets::{Clear, Paragraph, Wrap};

pub mod dashboard;
mod editing;
pub mod help;
mod yn;

impl Widget for &InputEntry {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_type(ratatui::widgets::BorderType::Plain);
        block.render(area, buf);
        Clear.render(area, buf);
        let name = self.about.as_str();
        let desc = self.notes.as_str();
        let identity = self.username.as_str();
        let password = self.password.as_str();
        let rc = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Fill(0),
            ])
            .split(area);

        let b_name = Block::bordered().title(" about ").fg(Color::White);
        let b_ident = Block::bordered().title(" username ").fg(Color::Red);
        let b_password = Block::bordered().title(" password ").fg(Color::Red);
        let b_description = Block::bordered().title(" notes ").fg(Color::White);

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

impl Widget for &NeedMainPwdState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::QuadrantOutside)
            .fg(Color::Red)
            .bg(Color::Red)
            .padding(Padding::horizontal(3));

        let block = if self.retry_count != 0 {
            let line = Line::from(format!(
                "VALID PASSWORD: ({}/{})",
                self.retry_count, ALLOC_VALID_MAIN_PASS_MAX
            ))
            .fg(Color::White);
            block.title_bottom(line.centered())
        } else {
            block
        };

        let inner_area = block.inner(area);

        block.render(area, buf);

        let rc_box_box = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(0),
                Constraint::Length(3),
                Constraint::Fill(0),
            ])
            .split(inner_area);

        let box_name = Block::default()
            .title("[󰌿] MAIN PASSWORD")
            .fg(Color::White)
            .borders(Borders::ALL);

        let i = self.mp_input.chars().count();
        let shard_v = "*".repeat(i);

        Paragraph::new(shard_v)
            .block(box_name)
            .alignment(Alignment::Center)
            .render(rc_box_box[1], buf);
    }
}

#[allow(dead_code)]
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

/// 光标闪烁状态
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(dead_code)]
pub enum Blink {
    None,
    Show,
    Hide,
}
/// 20250706：不应有该或其他手动实现，或应使用 cargo add tui-input 该 crate实现...
///  或参考 https://ratatui.rs/examples/apps/user_input/ 示例，但该示例因直接的字节间移动，导致实际不饿能处理unicode
///  字符...
/// ---
/// 有光标的输入框
/// 该实现尚未加入项目结构中，因为还找不到适合的结构去使用...
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InputField {
    content: String,        // 内容
    cursor_position: usize, // 光标位置
    blink: Blink,           // 光标闪烁状态
}
#[allow(dead_code)]
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

    /// 重置内容和光标位置，光标在内容后
    pub fn reset_content(&mut self, content: String) {
        self.content = content;
        self.cursor_position = self.content.len();
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
