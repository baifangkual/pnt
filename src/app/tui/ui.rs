use crate::app::consts::ALLOC_INVALID_MAIN_PASS_MAX;
use crate::app::entry::InputEntry;
use crate::app::tui::colors::{CL_BLACK, CL_BLUE, CL_D_WHITE, CL_L_BLACK, CL_LL_BLACK, CL_RED, CL_WHITE, CL_YELLOW, CL_DD_WHITE};
use crate::app::tui::components::Screen;
use crate::app::tui::components::states::VerifyMPHState;
use crate::app::tui::components::yn::YNState;
use crate::app::tui::ui::home_page::HomePageV1Widget;
use crate::app::tui::{TUIApp, layout};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Rect};
use ratatui::prelude::StatefulWidget;
use ratatui::prelude::{Color, Layout, Line, Modifier, Style, Stylize, Widget};
use ratatui::widgets::{Block, Borders, Padding};
use ratatui::widgets::{Clear, Paragraph, Wrap};
use tui_textarea::TextArea;

mod editing;
pub mod help;
pub mod home_page;

impl Widget for &mut TUIApp {
    /// 渲染函数入口
    ///
    /// ratatui的渲染逻辑是后渲染的覆盖先渲染的
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [middle, bottom] = Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]).areas(area);

        // 主要内容背景
        Block::new().bg(CL_BLACK).render(middle, buf);

        // 底部右边的mode
        let mut br_mode: Option<Paragraph> = None;
        // 显示mode的长度
        let mut mode_show_len = 0;

        // 渲染当前屏幕
        match &mut self.screen {
            Screen::HomePageV1(state) => {
                // find 框不为空，则右下提示当前生效
                if state.find_mode() || !state.current_find_input_is_empty() {
                    br_mode = Some(Paragraph::new("FIND").fg(CL_BLACK).bg(Color::Yellow));
                    mode_show_len = 6; // 增加左右
                }

                let dash_widget = HomePageV1Widget;
                dash_widget.render(middle, buf, state);
                // fixed: ratatui的ListState在未定义select时其值为uxx::MAX，增1溢出导致dev时panic
                // 遂应当在其stateful的渲染render完成后再读取其，便会有有效值了
                // 遂该块代码应在 dash_widget.render(middle, buf, state) 之后即可
                let cur = if let Some(i) = state.cursor_selected() {
                    i + 1
                } else {
                    0
                };
                self.state_info.clear();
                self.state_info.push_str(&format!(" {}/{}", cur, state.entry_count()));
            }
            Screen::Help(list_cursor) => {
                self.hot_msg.set_always_if_none("󰌌 <ESC>|<Q> back, ↓↑jk scroll");
                let rect = layout::centered_percent(90, 90, middle);
                let help_who = self.back_screen.last().unwrap(); // 一定有，遂直接unwrap
                match help_who {
                    Screen::HomePageV1(..) => help::HelpPage::home_page().render(rect, buf, list_cursor),
                    Screen::Details(..) => help::HelpPage::detail().render(rect, buf, list_cursor),
                    Screen::Edit(..) => help::HelpPage::editing().render(rect, buf, list_cursor),
                    _ => (),
                }
            }
            Screen::Details(entry, _) => {
                self.hot_msg
                    .set_always_if_none("󰌌 <ESC>|<Q> back, <E> edit, <C> CP, <D> delete, <L> relock");
                let rect = layout::centered_percent(90, 90, middle);
                entry.render(rect, buf);
            }
            Screen::Edit(state) => {
                self.hot_msg
                    .set_always_if_none("󰌌 <TAB> next, ↓↑←→ move, <CTRL+S> save, <ESC> back");
                let rect = layout::centered_percent(90, 90, middle);
                state.render(rect, buf);
                // 判定是新建还是编辑，右下提示
                if state.current_e_id().is_some() {
                    br_mode = Some(Paragraph::new("UPDATE").fg(CL_BLACK).bg(CL_YELLOW))
                } else {
                    br_mode = Some(Paragraph::new("CREATE").fg(CL_BLACK).bg(CL_BLUE));
                }
                mode_show_len = 8; // 增加左右空一格
            }
            Screen::YNOption(option_yn) => {
                self.hot_msg
                    .set_always_if_none("󰌌 <ENTER>|<Y> Yes, <ESC>|<N> No, ↓↑jk scroll");
                let rect = layout::centered_percent(70, 50, middle);
                option_yn.render(rect, buf);
            }
            Screen::InputMainPwd(state) => {
                let rect = layout::centered_fixed(50, 5, middle);
                state.render(rect, buf);
            }
        }

        // 页面右下角 当前/总共 entry 信息
        let bottom_right_state_info = self.state_info.as_str();

        // 对bottom 横条横向切分
        let [bl, bc, br1, br2_dyn] = Layout::horizontal([
            Constraint::Length(10),
            Constraint::Fill(0),
            Constraint::Length(mode_show_len),
            // dyn 特殊字符占多个字节，遂该值就是+2个字节，即能填充左右空格
            Constraint::Length(bottom_right_state_info.len() as u16),
        ])
        .areas(bottom);

        // 总体页面右下角区域，显示信息
        Paragraph::new(bottom_right_state_info)
            .fg(CL_BLACK)
            .bg(CL_DD_WHITE)
            .alignment(Alignment::Center)
            .render(br2_dyn, buf);

        // 有则渲染之
        if let Some(mode_span) = br_mode {
            mode_span.centered().render(br1, buf);
        }
        // bc 填充颜色
        Block::new().bg(CL_L_BLACK).render(bc, buf);

        // mp状态图标
        if self.context.is_verified() {
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

        // fixed 在match后渲染hot_msg，防止match内修改hot_msg后当前帧不刷新，而是下一帧刷新的问题
        // to do 后可作为当前screen 提示信息显示在此...
        Paragraph::new(self.hot_msg.msg())
            .alignment(self.hot_msg.alignment())
            .fg(self.hot_msg.color())
            .render(bc, buf);
    }
}

/// 选项页面渲染
impl Widget for &YNState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::new().bg(self.theme.cl_global_bg).padding(Padding::uniform(1));

        let inner_area = block.inner(area);

        block.render(area, buf);

        // desc 部分
        let box_desc = Block::new().bg(self.theme.cl_desc_bg).padding(Padding::proportional(1));

        /*
        https://docs.rs/ratatui/latest/ratatui/widgets/struct.Paragraph.html#method.line_count
        line_count稳定后，可计算占用行数从而动态分配 desc占用 area height大小
        let desc_paragraph = Paragraph::new(self.desc.as_str())
            .block(box_desc)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left)
            .fg(self.theme.cl_desc_fg);
        let desc_height = desc_paragraph.line_count(inner_area.width) as u16;
        desc_paragraph.render(r_desc, buf);
        */

        let [r_title, _, r_desc, _, r_bottom] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner_area);

        Paragraph::new(self.desc.as_str())
            .block(box_desc)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left)
            .fg(self.theme.cl_desc_fg)
            .scroll((self.scroll(), 0))
            .render(r_desc, buf);

        Paragraph::new(
            Line::from(self.title.as_str())
                .fg(self.theme.cl_title_fg)
                .bg(self.theme.cl_title_bg),
        )
        .alignment(Alignment::Center)
        .render(r_title, buf);

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

pub trait TextAreaExt {
    /// 设置 textarea 所谓 “激活状态”
    ///
    /// "激活的“ 光标可见，反之不可见
    fn set_activate_state(&mut self, state: bool);
}
impl TextAreaExt for TextArea<'_> {
    fn set_activate_state(&mut self, state: bool) {
        if state {
            // 光标处 反色，即显示光标
            self.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        } else {
            // 光标不可见
            self.set_cursor_style(Style::default());
        }
    }
}

/// 返回一个新的 tui_textarea::TextArea
pub fn new_input_textarea(place_holder_text: Option<&str>, activate_state: bool) -> TextArea<'_> {
    let mut textarea = TextArea::default();
    if let Some(place_holder_text) = place_holder_text {
        textarea.set_placeholder_text(place_holder_text);
    }
    textarea.set_cursor_line_style(Style::default());
    textarea.set_activate_state(activate_state);
    textarea
}

/// inputEntry直接的 渲染逻辑
impl Widget for &InputEntry {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_type(ratatui::widgets::BorderType::Plain);
        block.render(area, buf);
        Clear.render(area, buf);
        let name = self.about.as_str();
        let desc = self.notes.as_str();
        let identity = self.username.as_str();
        let password = self.password.as_str();
        let rc = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(0),
        ])
        .split(area);

        let b_name = Block::bordered().title(" 󰦨 about ").fg(CL_WHITE);
        let b_ident = Block::bordered().title(" 󰌿 username ").fg(CL_RED);
        let b_password = Block::bordered().title(" 󰌿 password ").fg(CL_RED);
        let b_description = Block::bordered().title(" 󰦨 notes ").fg(CL_WHITE);

        Paragraph::new(name).block(b_name).render(rc[0], buf);
        Paragraph::new(identity).block(b_ident).render(rc[1], buf);
        Paragraph::new(password).block(b_password).render(rc[2], buf);
        Paragraph::new(desc)
            // 虽然 detail直接切换到 edit notes显示过长的行部分会跳变
            // 但为了在detail时的信息完整性，允许跳变
            .wrap(Wrap { trim: false })
            .block(b_description)
            .render(rc[3], buf);
    }
}

/// 输入密码页的渲染逻辑
impl Widget for &VerifyMPHState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::QuadrantOutside)
            .fg(CL_RED)
            .bg(CL_RED)
            .padding(Padding::horizontal(3));

        let block = if self.retry_count != 0 {
            let line = Line::from(format!(
                " INVALID PASSWORD: ({}/{})",
                self.retry_count, ALLOC_INVALID_MAIN_PASS_MAX
            ))
            .fg(CL_WHITE);
            block.title_bottom(line.centered())
        } else {
            block
        };

        let inner_area = block.inner(area);

        block.render(area, buf);

        let rc_box_box = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(0), Constraint::Length(3), Constraint::Fill(0)])
            .split(inner_area);

        let box_name = Block::default()
            .title("[󰌿] MAIN PASSWORD")
            .fg(CL_WHITE)
            .borders(Borders::ALL);

        let i = self.mp_input.chars().count();
        let shard_v = "*".repeat(i);

        Paragraph::new(shard_v)
            .block(box_name)
            .alignment(Alignment::Center)
            .render(rc_box_box[1], buf);
    }
}
