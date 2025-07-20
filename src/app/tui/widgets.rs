use crate::app::consts::ALLOC_INVALID_MAIN_PASS_MAX;
use crate::app::entry::InputEntry;
use crate::app::tui::colors::{CL_RED, CL_WHITE};
use crate::app::tui::components::states::VerifyMPHState;
use crate::app::tui::components::yn::YNState;
use crate::app::tui::layout;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Rect};
use ratatui::prelude::{Layout, Line, Modifier, Style, Stylize, Widget};
use ratatui::widgets::{Block, Borders, Padding};
use ratatui::widgets::{Clear, Paragraph, Wrap};
use tui_textarea::TextArea;

mod editing;
pub mod help;
pub mod home_page;

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
            Line::from(format!(" [] {} ", self.title.as_str()))
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
