use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Color, Stylize, Widget};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use crate::app::tui::screen::states::{Editing, EditingState};

impl Widget for &EditingState {

    fn render(self, area: Rect, buf: &mut Buffer) {
        let input_entry = self.current_input_entry();
        let curr_editing = self.current_editing_type();

        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::Plain);
        block.render(area, buf);
        Clear.render(area, buf);
        let name = input_entry.name.as_str();
        let desc = input_entry.description.as_str();
        let identity = input_entry.identity.as_str();
        let password = input_entry.password.as_str();
        let rc = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Fill(0),
            ])
            .split(area);

        let mut b_name = Block::bordered().title("name").fg(Color::White);
        let mut b_ident = Block::bordered().title("identity").fg(Color::White);
        let mut b_password = Block::bordered().title("password").fg(Color::White);
        let mut b_description = Block::bordered().title("description").fg(Color::White);

        match curr_editing {
            Editing::Name => {b_name = b_name.fg(Color::Yellow)}
            Editing::Identity => {b_ident = b_ident.fg(Color::Yellow)}
            Editing::Password => {b_password = b_password.fg(Color::Yellow)}
            Editing::Description => {b_description = b_description.fg(Color::Yellow)}
        }
        Paragraph::new(name).block(b_name).render(rc[0], buf);
        Paragraph::new(identity).block(b_ident).render(rc[1], buf);
        Paragraph::new(password).block(b_password).render(rc[2], buf);
        Paragraph::new(desc).wrap(Wrap{trim: false}).block(b_description).render(rc[3], buf);
    }
}