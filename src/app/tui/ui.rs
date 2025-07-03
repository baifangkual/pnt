use super::runtime::TUIRuntime;
use crate::app::tui::screen::Screen;
use ratatui::layout::Direction;
use ratatui::prelude::{Constraint, Layout, Line, Span, Style};
use ratatui::widgets::{List, ListItem};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};

const TITLE: &str = concat!(
    clap::crate_name!(),
    "-v",
    clap::crate_version!(),
    "-",
    "help:F1"
);

impl Widget for &TUIRuntime {
    /// 渲染函数入口
    /// ratatui的渲染逻辑是后渲染的覆盖先渲染的
    /// 遂该方法内始终先渲染 dashboard
    /// 再渲染当前的 screen
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 当 back_screen 有值，一定为 dash，渲染之
        // 若无，则证明当前为 dash，则 if 内 不渲染
        if let Some(dash) = self.back_screen.get(0) {
            dash.render(area, buf);
        }
        // 渲染当前屏幕
        self.screen.render(area, buf);
    }
}

impl Widget for &Screen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Screen::Dashboard { cursor, entries } => {
                // 外壳
                let block = Block::bordered()
                    .title(Line::from(TITLE).style(Style::default().fg(Color::Black)
                        .bg(Color::White)))
                    .fg(Color::White)
                    .border_type(BorderType::Plain); // plain 无圆角边框
                // 渲染之
                block.render(area, buf);

                let layout_1f1 = Layout::default()
                    .direction(Direction::Horizontal) // 水平
                    .constraints([
                        Constraint::Max(5), Constraint::Percentage(90), Constraint::Max(5),
                    ])
                    .split(area);

                let layout_hm1 = Layout::default()
                    .direction(Direction::Vertical) // 垂直
                    .constraints([
                        Constraint::Min(5),
                        Constraint::Percentage(93),
                        Constraint::Min(2),
                    ])
                    .split(layout_1f1[1]);

                // list

                let r_l = if let Some(c_ptr) = cursor {
                    // 有元素
                    let mut list_items = Vec::<ListItem>::new();
                    for (i, ent) in entries.iter().enumerate() {
                        let li = if i == *c_ptr {
                            // 光标所在
                            ListItem::new(Line::from(Span::styled( // todo 显示 格式化
                                format!("{: >3} : {}", i, ent.name),
                                Style::default().fg(Color::Black).bg(Color::White),
                            )))
                        } else {
                            ListItem::new(Line::from(Span::styled( // todo 显示 格式化
                                format!("{: >3} : {}", i, ent.name),
                                Style::default().fg(Color::White),
                            )))
                        };
                        list_items.push(li)
                    }
                    List::new(list_items)
                } else {
                    // 无元素
                    List::new::<[ListItem; 0]>([])
                };

                // 渲染之
                r_l.render(layout_hm1[1], buf);
            }
            Screen::Help => {}
            Screen::Details(entry) => {}
            Screen::Creating { editing, u_input } => {}
            Screen::DeleteTip(_, name, desc) => {}
            Screen::Updating {
                editing,
                u_input,
                e_id,
            } => {}
            Screen::NeedMainPasswd(ip, _, re_try) => {}
        }
    }
}
