use crate::app::tui::screen::states::EditingState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Color, Stylize, Widget};
use ratatui::widgets::{Block, Clear, Paragraph};

impl Widget for &EditingState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let areas: [Rect; 5] = Layout::vertical([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Fill(0),   // notes
            ratatui::layout::Constraint::Length(1), // 模糊的按键提示
        ])
        .areas(area);

        let input_entry = self.current_input_entry();
        let curr_editing = self.current_editing_type();
        let about = input_entry.about.as_str();
        let notes = input_entry.notes.as_str();
        let username = input_entry.username.as_str();
        let password = input_entry.password.as_str();

        // 未填写情况下 添加 * 前缀
        let title_name = if about.is_empty() {
            " (*) 󰦨 about "
        } else {
            " 󰦨 about "
        };
        let title_ident = if username.is_empty() {
            " (*) 󰌿 username "
        } else {
            " 󰌿 username "
        };
        let title_password = if password.is_empty() {
            " (*) 󰌿 password "
        } else {
            " 󰌿 password "
        };

        let b_about = Block::bordered().title(title_name).fg(Color::White);
        let b_username = Block::bordered().title(title_ident).fg(Color::White);
        let b_password = Block::bordered().title(title_password).fg(Color::White);
        let b_notes = Block::bordered().title(" 󰦨 notes ").fg(Color::White);

        let mut blocks = [Some(b_about), Some(b_username), Some(b_password), Some(b_notes)];
        let paragraph_text = [about, username, password, notes];

        for idx in 0..4_usize {
            let mut blc = blocks[idx].take().unwrap();
            let curr_area = areas[idx];

            if idx == curr_editing.index() {
                // is_active
                // 正在编辑的，fg yellow，光标显示
                blc = blc.fg(Color::Yellow);
                let inner = blc.inner(curr_area);
                blc.render(curr_area, buf);
                self.textarea(*curr_editing).render(inner, buf);
            } else {
                // 非正在编辑的...
                let text = paragraph_text[idx];
                if idx < 3 && text.is_empty() {
                    // 非notes且没有值的，提示其必须
                    Paragraph::new(" require") // 一个空格在前，和有光标但空行对齐
                        .fg(Color::DarkGray)
                        .block(blc)
                        .render(curr_area, buf);
                } else {
                    Paragraph::new(paragraph_text[idx]).block(blc).render(curr_area, buf);
                }
            }
        }
    }
}
