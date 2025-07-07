use super::rt::TUIApp;
use crate::app::tui::screen::Screen;
use ratatui::prelude::Line;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    widgets::{Block, BorderType, StatefulWidget, Widget},
};
use crate::app::tui::widgets::dashboard::DashboardWidget;
use crate::app::tui::widgets::help;
use crate::app::tui::layout;


const TITLE: &str = concat!(
clap::crate_name!(),
"-v",
clap::crate_version!(),
"-",
"help:F1"
);

/// 深灰色背景
const TUI_BG_COLOR: Color = Color::from_u32(0x252624);
/// title color
const TUI_TITLE_COLOR: Color = Color::from_u32(0x4D4F4B);


impl Widget for &mut TUIApp {
    /// 渲染函数入口
    /// ratatui的渲染逻辑是后渲染的覆盖先渲染的
    /// 遂该方法内始终先渲染 dashboard
    /// 再渲染当前的 screen
    fn render(self, area: Rect, buf: &mut Buffer) {

        let block = Block::bordered()
            .title(Line::from(TITLE).fg(TUI_TITLE_COLOR))
            .fg(TUI_BG_COLOR) // 灰色填充
            .bg(TUI_BG_COLOR)
            .border_type(BorderType::QuadrantOutside);
        // 创建内容区域
        let inner_area = block.inner(area);
        block.render(area, buf);


        // 当 back_screen 有值，一定为 dash，渲染之
        // 若无，则证明当前为 dash，则 if 内 不渲染
        // 注释该，仅渲染 当前 screen
        // if let Some( Screen::Dashboard(state)) = self.back_screen.get_mut(0) {
        //     let dash_widget = DashboardWidget;
        //     dash_widget.render(inner_area, buf, state)
        // }

        // 渲染当前屏幕
        match &mut self.screen {
            Screen::Dashboard(state) => {
                let dash_widget = DashboardWidget;
                dash_widget.render(inner_area, buf, state)
            }
            Screen::Help => {
                let rect = layout::centered_rect(90, 90, inner_area);
                help::HELP_PAGE_DASHBOARD.render(rect, buf)
            }
            Screen::Details(entry, _) => {
                let rect = layout::centered_rect(90, 90, inner_area);
                entry.render(rect, buf);
            }
            Screen::Edit(state) => {
                let rect = layout::centered_rect(90, 90, inner_area);
                state.render(rect, buf);
            }
            Screen::DeleteTip(option_yn) => {
                let rect = layout::centered_rect(70, 60, inner_area);
                option_yn.render(rect, buf);
            }
            Screen::NeedMainPasswd(state) => {
                let rect = layout::centered_rect(50, 20, inner_area);
                state.render(rect, buf);
            }
        }
    }
}
