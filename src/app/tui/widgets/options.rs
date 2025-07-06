use crate::app::entry::EncryptedEntry;
use crate::app::tui::layout;
use crate::app::tui::layout::RectExt;
use crate::app::tui::screen::options::OptionYN;
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Line, Stylize, Widget};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};


/// 删除前提示
impl Widget for &OptionYN<EncryptedEntry> {
    fn render(self, area: Rect, buf: &mut Buffer) {

        Clear.render(area, buf);

        let block_color = Color::from_u32(0x220000);

        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::QuadrantOutside)
            .fg(block_color)
            .bg(block_color);

        let inner_area = block.inner(area);

        block.render(area, buf);


        let rc_box_box = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Fill(0),
                Constraint::Length(3),
                Constraint::Percentage(60),
                Constraint::Fill(0),
                Constraint::Length(1),
            ])
            .split(inner_area);

        Paragraph::new(Line::from(format!(" {} ",self.title.as_str()))
            .fg(Color::White).bg(Color::Red))
            .alignment(Alignment::Center)
            .render(rc_box_box[0], buf);

        let box_name = Block:: default()
            .title("name")
            .fg(Color::White)
            .borders(Borders::ALL);

        Paragraph::new(self.content().unwrap().name.as_str())
            .block(box_name)
            .alignment(Alignment::Left)
            .render(rc_box_box[2].h_centered_rect(90), buf);

        let box_desc = Block::default()
            .title("description")
            .fg(Color::White)
            .borders(Borders::ALL);

        Paragraph::new(self.desc.as_str())
           .block(box_desc)
            .wrap(Wrap{trim: false})
            .alignment(Alignment::Left)
           .render(rc_box_box[3].h_centered_rect(90), buf);

        // 底部左右二分
        let rc_bottom_lr = layout::split_lr_rects(rc_box_box[5]);
        // 底部 YN
        Paragraph::new(Line::from("[(Y)es]").centered().bg(Color::Red).fg(Color::White))
            .alignment(Alignment::Center)
            .render(rc_bottom_lr[0], buf);
        Paragraph::new(Line::from("[(N)o]").centered().bg(Color::Red).fg(Color::White))
            .alignment(Alignment::Center)
            .render(rc_bottom_lr[1], buf);
    }
}