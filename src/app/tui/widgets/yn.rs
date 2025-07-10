use crate::app::tui::layout;
use crate::app::tui::screen::yn::YNState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Alignment, Constraint, Line, Stylize, Widget};
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Wrap};

/// 删除前提示
impl Widget for &YNState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::new().bg(self.theme.cl_global_bg).padding(Padding::uniform(1));

        let inner_area = block.inner(area);

        block.render(area, buf);

        let [r_title, _, r_desc, _, r_bottom] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner_area);

        Paragraph::new(
            Line::from(format!(" [] {} ", self.title.as_str()))
                .fg(self.theme.cl_title_fg)
                .bg(self.theme.cl_title_bg),
        )
        .alignment(Alignment::Center)
        .render(r_title, buf);

        // desc 部分
        let box_desc = Block::new().bg(self.theme.cl_desc_bg).padding(Padding::proportional(1));
        Paragraph::new(self.desc.as_str())
            .block(box_desc)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left)
            .fg(self.theme.cl_desc_fg)
            .render(r_desc, buf);

        // 底部左右二分
        let rc_bottom_lr = layout::horizontal_split2(r_bottom);
        // 底部 YN
        Paragraph::new(
            Line::from("[ (Y)es ]")
                .centered()
                .bg(self.theme.cl_y_bg)
                .fg(self.theme.cl_y_fg),
        )
        .alignment(Alignment::Center)
        .render(rc_bottom_lr[0], buf);
        Paragraph::new(
            Line::from("[ (N)o ]")
                .centered()
                .bg(self.theme.cl_n_bg)
                .fg(self.theme.cl_n_fg),
        )
        .alignment(Alignment::Center)
        .render(rc_bottom_lr[1], buf);
    }
}
