use crate::app::tui::layout;
use crate::app::tui::layout::RectExt;
use crate::app::tui::screen::options::YNState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Line, Stylize, Widget};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

const DELETE_TIP_BG_COLOR: Color = Color::from_u32(0x220000);
const DELETE_TIP_DESC_BG_COLOR: Color = Color::from_u32(0x1E0000);

/// 删除前提示
impl Widget for &YNState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::QuadrantOutside)
            .fg(DELETE_TIP_BG_COLOR)
            .bg(DELETE_TIP_BG_COLOR);

        let inner_area = block.inner(area);

        block.render(area, buf);

        let rc_box_box = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Fill(0),
                Constraint::Percentage(60),
                Constraint::Fill(0),
                Constraint::Length(1),
            ])
            .split(inner_area);

        Paragraph::new(Line::from(format!(" {} ",self.title.as_str()))
            .fg(Color::White).bg(Color::Red))
            .alignment(Alignment::Center)
            .render(rc_box_box[0], buf);

        let box_desc = Block::default()
            .fg(DELETE_TIP_DESC_BG_COLOR)
            .bg(DELETE_TIP_DESC_BG_COLOR)
            .borders(Borders::ALL);

        Paragraph::new(self.desc.as_str())
           .block(box_desc)
            .wrap(Wrap{trim: false})
            .alignment(Alignment::Left)
            .fg(Color::White)
           .render(rc_box_box[2].h_centered_rect(90), buf);

        // 底部左右二分
        let rc_bottom_lr = layout::split_lr_rects(rc_box_box[4]);
        // 底部 YN
        Paragraph::new(Line::from("[(Y)es]").centered().bg(Color::Red).fg(Color::White))
            .alignment(Alignment::Center)
            .render(rc_bottom_lr[0], buf);
        Paragraph::new(Line::from("[(N)o]").centered().bg(Color::Red).fg(Color::White))
            .alignment(Alignment::Center)
            .render(rc_bottom_lr[1], buf);
    }
}