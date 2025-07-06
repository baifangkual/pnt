use std::ops::Deref;
use super::runtime::TUIRuntime;
use crate::app::tui::screen::Screen;
use ratatui::layout::Direction;
use ratatui::prelude::{Constraint, Layout, Line, Span, Style};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Widget},
};
use crate::app::tui::widgets::dashboard::DashboardWidget;
use crate::app::tui::widgets::help;
use crate::app::tui::widgets::help::HelpPage;
use crate::app::tui::util;

impl Widget for &mut TUIRuntime {
    /// 渲染函数入口
    /// ratatui的渲染逻辑是后渲染的覆盖先渲染的
    /// 遂该方法内始终先渲染 dashboard
    /// 再渲染当前的 screen
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 当 back_screen 有值，一定为 dash，渲染之
        // 若无，则证明当前为 dash，则 if 内 不渲染
        if let Some( Screen::Dashboard(state)) = self.back_screen.get_mut(0) {
            let dash_widget = DashboardWidget;
            dash_widget.render(area, buf, state)
        }
        // 渲染当前屏幕
        match &mut self.screen {
            Screen::Dashboard(state) => {
                let dash_widget = DashboardWidget;
                dash_widget.render(area, buf, state)
            }
            Screen::Help => {
                help::HELP_PAGE_DASHBOARD.render(area, buf)
            }
            Screen::Details(entry) => {
                let rect = util::centered_rect(90, 90, area);
                entry.render(rect, buf);
            }
            Screen::Creating(state) => {
                let rect = util::centered_rect(90, 90, area);
                state.render(rect, buf);
            }
            Screen::DeleteTip(option_yn) => {
                let rect = util::centered_rect(80, 70, area);
                option_yn.render(rect, buf);
            }
            Screen::Updating(state) => {
                let rect = util::centered_rect(90, 90, area);
                state.render(rect, buf);
            }
            Screen::NeedMainPasswd(state) => {}
        }
    }
}
