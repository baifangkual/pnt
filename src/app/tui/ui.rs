
use crate::app::tui::colors::{CL_BLACK, CL_DD_WHITE, CL_L_BLACK, CL_LL_BLACK, CL_RED, CL_WHITE};
use crate::app::tui::{layout, TUIApp};
use crate::app::tui::screen::Screen;
use crate::app::tui::widgets::dashboard::DashboardWidget;
use crate::app::tui::widgets::help;
use ratatui::layout::Alignment;
use ratatui::prelude::{Constraint, Layout};
use ratatui::widgets::Paragraph;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Block, StatefulWidget, Widget},
};

const TITLE: &str = concat!(clap::crate_name!(), " v", clap::crate_version!(), " ?:<f1>");

impl Widget for &mut TUIApp {
    /// 渲染函数入口
    ///
    /// ratatui的渲染逻辑是后渲染的覆盖先渲染的
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, middle, bottom] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(0), Constraint::Length(1)]).areas(area);

        Paragraph::new(TITLE)
            .fg(CL_DD_WHITE)
            .block(Block::new().bg(CL_L_BLACK))
            .render(top, buf);
        // 主要内容背景
        Block::new().bg(CL_BLACK).render(middle, buf);

        let [bl, bc, br] =
            Layout::horizontal([Constraint::Length(10), Constraint::Fill(0), Constraint::Length(7)]).areas(bottom);

        // bc 填充颜色
        // todo 后可作为当前screen 提示信息显示在此...
        Block::new().bg(CL_L_BLACK).render(bc, buf);

        // mp状态图标
        if self.pnt.is_verified() {
            Paragraph::new("󰌾 UNLOCK")
                .fg(CL_WHITE)
                .bg(CL_RED)
                .alignment(Alignment::Center)
                .render(bl, buf);
        } else {
            Paragraph::new("󰌾 LOCK")
                .fg(CL_WHITE)
                .bg(CL_LL_BLACK)
                .alignment(Alignment::Center)
                .render(bl, buf);
        }
        // 多少个条目
        Paragraph::new(format!(" {}", self.store_entry_count))
            .fg(CL_WHITE)
            .bg(CL_LL_BLACK)
            .alignment(Alignment::Center)
            .render(br, buf);

        // 渲染当前屏幕
        match &mut self.screen {
            Screen::DashboardV1(state) => {
                let dash_widget = DashboardWidget;
                dash_widget.render(middle, buf, state)
            }
            Screen::Help => {
                let rect = layout::centered_percent(90, 90, middle);
                help::HelpPage::HELP_PAGE_DASHBOARD.render(rect, buf)
            }
            Screen::Details(entry, _) => {
                let rect = layout::centered_percent(90, 90, middle);
                entry.render(rect, buf);
            }
            Screen::Edit(state) => {
                let rect = layout::centered_percent(90, 90, middle);
                state.render(rect, buf);
            }
            Screen::YNOption(option_yn) => {
                let rect = layout::centered_percent(70, 50, middle);
                option_yn.render(rect, buf);
            }
            Screen::NeedMainPasswd(state) => {
                let rect = layout::centered_fixed(50, 5, middle);
                state.render(rect, buf);
            }
        }
    }
}
