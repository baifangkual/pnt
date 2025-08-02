use crate::app::tui::colors::{CL_WHITE, CL_YELLOW};
use crate::app::tui::components::states::EditingState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Stylize, Widget};
use ratatui::widgets::{Block, Clear};

impl Widget for &EditingState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let areas: [Rect; 4] = Layout::vertical([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Fill(0), // notes
        ])
        .areas(area);

        let curr_editing = self.current_editing_type();
        let all_textarea = self.all_textarea();

        // 未填写情况下 添加 * 前缀
        let title_name = if all_textarea[0].is_empty() {
            " (*) 󰦨 about "
        } else {
            " 󰦨 about "
        };
        let title_ident = if all_textarea[1].is_empty() {
            " (*) 󰌿 username "
        } else {
            " 󰌿 username "
        };
        let title_password = if all_textarea[2].is_empty() {
            " (*) 󰌿 password "
        } else {
            " 󰌿 password "
        };

        let b_about = Block::bordered().title(title_name).fg(CL_WHITE);
        let b_username = Block::bordered().title(title_ident).fg(CL_WHITE);
        let b_password = Block::bordered().title(title_password).fg(CL_WHITE);
        let b_notes = Block::bordered().title(" 󰦨 notes ").fg(CL_WHITE);

        let mut blocks = [
            Some(b_about),
            Some(b_username),
            Some(b_password),
            Some(b_notes),
        ];

        for idx in 0..4_usize {
            let blc = blocks[idx].take().unwrap();
            let curr_area = areas[idx];
            let n_blc = if idx == curr_editing as usize {
                // is_active
                // 正在编辑的，fg yellow，光标显示
                blc.fg(CL_YELLOW)
            } else {
                // 非正在编辑的...
                // fixed 修复因 notes太多时，因焦点切换，
                // notes内容使用Paragraph渲染时与textarea渲染在光标在下方行时显示内容不一致问题
                // 都统一使用textarea渲染... textarea光标的显示与否可通过
                // textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
                // 进行切换... 参考自：https://github.com/rhysd/tui-textarea/blob/HEAD/examples/split.rs
                blc
            };
            let inner = n_blc.inner(curr_area);
            n_blc.render(curr_area, buf);
            all_textarea[idx].render(inner, buf);
        }
    }
}
