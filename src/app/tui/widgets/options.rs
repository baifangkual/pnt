use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Line, Stylize, Widget};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};
use crate::app::entry::EncryptedEntry;
use crate::app::tui::screen::options::OptionYN;

impl Widget for &OptionYN<EncryptedEntry> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized
    {

        Clear.render(area, buf);

        let block = Block::bordered()
            .title(Line::from(format!(" {} ",self.title.as_str()))
                .fg(Color::Black).bg(Color::White)
                .centered())
            .border_type(ratatui::widgets::BorderType::Plain)
            .title_bottom(Line::from("[(Y)es]").centered().bg(Color::Red).fg(Color::White))
            .title_bottom(Line::from("       OR       ").centered())
            .title_bottom(Line::from("[(N)o]").centered().bg(Color::Red).fg(Color::White))
            .padding(Padding::uniform(3));

        let inner_area = block.inner(area);

        block.render(area, buf);


        let rc_box_box = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Percentage(60),
            ])
            .split(inner_area);

        // Paragraph::new(Line::from(format!(" {} ",self.title.as_str()))
        //     .fg(Color::Black).bg(Color::White)
        //     .centered())
        //     .alignment(Alignment::Left)
        //     .render(rc_box_box[0], buf);

        let box_name = Block:: default()
            .title("name")
            .borders(Borders::ALL);

        Paragraph::new(self.content().unwrap().name.as_str())
            .block(box_name)
            .alignment(Alignment::Left)
            .render(rc_box_box[1], buf);

        let box_desc = Block::default()
            .title("description")
            .borders(Borders::ALL);

        Paragraph::new(self.desc.as_str())
           .block(box_desc)
            .wrap(Wrap{trim: false})
            .alignment(Alignment::Left)
           .render(rc_box_box[2], buf);
    }
}