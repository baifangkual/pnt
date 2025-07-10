use super::rt::TUIApp;
use crate::app::tui::layout;
use crate::app::tui::layout::RectExt;
use crate::app::tui::screen::Screen;
use crate::app::tui::widgets::dashboard::DashboardWidget;
use crate::app::tui::widgets::help;
use ratatui::prelude::Line;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    widgets::{Block, StatefulWidget, Widget},
};
use ratatui::layout::Alignment;
use ratatui::widgets::Padding;
use crate::app::tui::colors::{CL_GLOBAL_BG, CL_GLOBAL_TITLE, CL_RED, CL_WHITE};

const TITLE: &str = concat!(clap::crate_name!(), " v", clap::crate_version!(), " ?:<f1>");

impl Widget for &mut TUIApp {
    /// 渲染函数入口
    ///
    /// ratatui的渲染逻辑是后渲染的覆盖先渲染的
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::from(TITLE).fg(CL_GLOBAL_TITLE))
            .bg(CL_GLOBAL_BG)
            .padding(Padding::bottom(1)); // 底部inner一行

        // 创建内容区域
        let inner_area = block.inner(area);
        block.render(area, buf);

        // todo 底部状态栏... horizontal_split::<10>() 不应
        //  该是长条...
        let bottom_rect = area.bottom_rect();
        let bn = bottom_rect.horizontal_split::<10>();

        // mp状态图标
        if self.pnt.is_verified() {
            ratatui::widgets::Paragraph::new("󰌾 UNLOCK")
                .fg(CL_WHITE).bg(CL_RED)
                .alignment(Alignment::Center)
                .render(bn[0], buf);
        } else {
            ratatui::widgets::Paragraph::new("󰌾 LOCK")
                .fg(CL_WHITE).bg(CL_GLOBAL_TITLE)
                .alignment(Alignment::Center)

                .render(bn[0], buf);
        }

        // 渲染当前屏幕
        match &mut self.screen {
            Screen::DashboardV1(state) => {
                let dash_widget = DashboardWidget;
                dash_widget.render(inner_area, buf, state)
            }
            Screen::Help => {
                let rect = layout::centered_percent(90, 90, inner_area);
                help::HELP_PAGE_DASHBOARD.render(rect, buf)
            }
            Screen::Details(entry, _) => {
                let rect = layout::centered_percent(90, 90, inner_area);
                entry.render(rect, buf);
            }
            Screen::Edit(state) => {
                let rect = layout::centered_percent(90, 90, inner_area);
                state.render(rect, buf);
            }
            Screen::YNOption(option_yn) => {
                let rect = layout::centered_percent(70, 50, inner_area);
                option_yn.render(rect, buf);
            }
            Screen::NeedMainPasswd(state) => {
                let rect = layout::centered_fixed(50, 5, inner_area);
                state.render(rect, buf);
            }
        }
    }
}
