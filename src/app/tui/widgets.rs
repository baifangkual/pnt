use crate::app::consts::ALLOC_VALID_MAIN_PASS_MAX;
use crate::app::entry::InputEntry;
use crate::app::tui::screen::states::NeedMainPwdState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Rect};
use ratatui::prelude::{Color, Layout, Line, Style, Stylize, Widget};
use ratatui::widgets::{Block, Borders, Padding};
use ratatui::widgets::{Clear, Paragraph, Wrap};
use tui_textarea::TextArea;

pub mod dashboard;
mod editing;
pub mod help;
mod yn;


/// 返回一个新的 tui_textarea::TextArea
pub fn new_input_textarea(place_holder_text: Option<&str>) -> TextArea<'_> {
    let mut textarea = TextArea::default();
    if let Some(place_holder_text) = place_holder_text {
        textarea.set_placeholder_text(place_holder_text);
    }
    textarea.set_cursor_line_style(Style::default());
    textarea
}

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

        let b_name = Block::bordered().title(" 󰦨 about ").fg(Color::White);
        let b_ident = Block::bordered().title(" 󰌿 username ").fg(Color::Red);
        let b_password = Block::bordered().title(" 󰌿 password ").fg(Color::Red);
        let b_description = Block::bordered().title(" 󰦨 notes ").fg(Color::White);

        Paragraph::new(name).block(b_name).render(rc[0], buf);
        Paragraph::new(identity).block(b_ident).render(rc[1], buf);
        Paragraph::new(password).block(b_password).render(rc[2], buf);
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
            .constraints(vec![Constraint::Fill(0), Constraint::Length(3), Constraint::Fill(0)])
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
