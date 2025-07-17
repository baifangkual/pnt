use crate::app::tui::colors::{CL_BLACK, CL_DD_WHITE, CL_L_BLACK, CL_LL_BLACK, CL_RED, CL_WHITE, CL_DDD_WHITE};
use crate::app::tui::screen::Screen;
use crate::app::tui::widgets::help;
use crate::app::tui::widgets::home_page::HomePageV1Widget;
use crate::app::tui::{TUIApp, layout};
use ratatui::layout::Alignment;
use ratatui::prelude::{Constraint, Layout};
use ratatui::widgets::Paragraph;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Block, StatefulWidget, Widget},
};



impl Widget for &mut TUIApp {
    /// 渲染函数入口
    ///
    /// ratatui的渲染逻辑是后渲染的覆盖先渲染的
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [middle, bottom] =
            Layout::vertical([ Constraint::Fill(0), Constraint::Length(1)]).areas(area);

        // 主要内容背景
        Block::new().bg(CL_BLACK).render(middle, buf);

        let [bl, bc, br] =
            Layout::horizontal([Constraint::Length(10), Constraint::Fill(0), Constraint::Length(7)]).areas(bottom);

        // bc 填充颜色
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
            Screen::HomePageV1(state) => {
                let dash_widget = HomePageV1Widget;
                dash_widget.render(middle, buf, state)
            }
            Screen::Help(list_cursor) => {
                self.hot_msg.set_if_not_always("󰌌 <ESC>|<Q> quit-help");
                let rect = layout::centered_percent(90, 90, middle);
                let help_who = self.back_screen.last().unwrap(); // 一定有，遂直接unwrap
                match help_who {
                    Screen::HomePageV1(..) => help::HelpPage::home_page().render(rect, buf, list_cursor),
                    Screen::Details(..) => help::HelpPage::detail().render(rect, buf, list_cursor),
                    Screen::Edit(..) => help::HelpPage::editing().render(rect, buf, list_cursor),
                    _ => ()
                }
            }
            Screen::Details(entry, _) => {
                self.hot_msg.set_if_not_always("󰌌 <ESC>|<Q> quit-detail , <D> delete , <L> quit-detail and re-lock");
                let rect = layout::centered_percent(90, 90, middle);
                entry.render(rect, buf);
            }
            Screen::Edit(state) => {
                self.hot_msg.set_if_not_always("󰌌 <TAB> next , ↓↑←→ move , <CTRL+S> save , <ESC> quit-edit");
                let rect = layout::centered_percent(90, 90, middle);
                state.render(rect, buf);
            }
            Screen::YNOption(option_yn) => {
                self.hot_msg.set_if_not_always("󰌌 <ENTER>|<Y> Yes , <ESC>|<N> No");
                let rect = layout::centered_percent(70, 50, middle);
                option_yn.render(rect, buf);
            }
            Screen::NeedMainPasswd(state) => {
                let rect = layout::centered_fixed(50, 5, middle);
                state.render(rect, buf);
            }
        }

        // fixed 在match后渲染hot_msg，防止match内修改hot_msg后当前帧不刷新，而是下一帧刷新的问题
        // to do 后可作为当前screen 提示信息显示在此...
        let hot_tip_msg = Paragraph::new(self.hot_msg.msg())
            .alignment(self.hot_msg.msg_alignment());
        if self.hot_msg.is_temp_msg() {
            hot_tip_msg.fg(CL_DDD_WHITE).render(bc, buf)
        } else {
            hot_tip_msg.fg(CL_DD_WHITE).render(bc, buf)
        }

    }
}
