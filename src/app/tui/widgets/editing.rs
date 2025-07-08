use crate::app::tui::screen::states::{Editing, EditingState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Color, Stylize, Widget};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use tui_textarea::TextArea;

struct FieldRenderParams<'a> {
    block: Block<'a>,
    content: &'a str,
    area: Rect,
    is_active: bool,
    textarea: &'a TextArea<'static>,
}


impl Widget for &EditingState {
    fn render(self, area: Rect, buf: &mut Buffer) {


        // let block = Block::bordered().border_type(ratatui::widgets::BorderType::Plain);
        // block.render(area, buf);

        Clear.render(area, buf);


        let [r0, r1, r2, r3, r4] = Layout::vertical(
            [
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Fill(0), // notes
                ratatui::layout::Constraint::Length(1), // 模糊的按键提示
            ]
        ).areas(area);

        let input_entry = self.current_input_entry();
        let curr_editing = self.current_editing_type();
        let about = input_entry.about.as_str();
        let notes = input_entry.notes.as_str();
        let username = input_entry.username.as_str();
        let password = input_entry.password.as_str();

        // 未填写情况下 添加 * 前缀
        let title_name = if about.is_empty() {
            " (*) 󰦨 about "
        } else {
            " 󰦨 about "
        };
        let title_ident = if username.is_empty() {
            " (*) 󰌿 username "
        } else {
            " 󰌿 username "
        };
        let title_password = if password.is_empty() {
            " (*) 󰌿 password "
        } else {
            " 󰌿 password "
        };

        let mut b_about = Block::bordered().title(title_name).fg(Color::White);
        let mut b_username = Block::bordered().title(title_ident).fg(Color::White);
        let mut b_password = Block::bordered().title(title_password).fg(Color::White);
        let mut b_notes = Block::bordered().title(" 󰦨 notes ").fg(Color::White);

        // match curr_editing {
        //     Editing::About => {
        //         b_about = b_about.fg(Color::Yellow);
        //         let inner = b_about.inner(r0);
        //         b_about.render(r0, buf);
        //         self.textarea(Editing::About).render(inner, buf);
        //         Paragraph::new(username).block(b_username).render(r1, buf);
        //         Paragraph::new(password).block(b_password).render(r2, buf);
        //         Paragraph::new(notes)
        //             .wrap(Wrap { trim: false })
        //             .block(b_notes)
        //             .render(r3, buf);
        //     },
        //     Editing::Username => {
        //         b_username = b_username.fg(Color::Yellow);
        //         let inner = b_username.inner(r1);
        //         b_username.render(r1, buf);
        //         self.textarea(Editing::Username).render(inner, buf);
        //         Paragraph::new(about).block(b_about).render(r0, buf);
        //         Paragraph::new(password).block(b_password).render(r2, buf);
        //         Paragraph::new(notes)
        //             .wrap(Wrap { trim: false })
        //             .block(b_notes)
        //             .render(r3, buf);
        //     },
        //     Editing::Password => {
        //         b_password = b_password.fg(Color::Yellow);
        //         let inner = b_password.inner(r2);
        //         b_password.render(r2, buf);
        //         self.textarea(Editing::Password).render(inner, buf);
        //         Paragraph::new(about).block(b_about).render(r0, buf);
        //         Paragraph::new(username).block(b_username).render(r1, buf);
        //         Paragraph::new(notes)
        //             .wrap(Wrap { trim: false })
        //             .block(b_notes)
        //             .render(r3, buf);
        //     },
        //     Editing::Notes => {
        //         b_notes = b_notes.fg(Color::Yellow);
        //         let inner = b_notes.inner(r3);
        //         b_notes.render(r3, buf);
        //         self.textarea(Editing::Notes).render(inner, buf);
        //         Paragraph::new(about).block(b_about).render(r0, buf);
        //         Paragraph::new(username).block(b_username).render(r1, buf);
        //         Paragraph::new(password).block(b_password).render(r2, buf);
        //     },
        // }

        let fields = [
            FieldRenderParams {
                block: b_about,
                content: about,
                area: r0,
                is_active: *curr_editing == Editing::About,
                textarea: self.textarea(Editing::About),
            },
            FieldRenderParams {
                block: b_username,
                content: username,
                area: r1,
                is_active: *curr_editing == Editing::Username,
                textarea: self.textarea(Editing::Username),
            },
            FieldRenderParams {
                block: b_password,
                content: password,
                area: r2,
                is_active: *curr_editing == Editing::Password,
                textarea: self.textarea(Editing::Password),
            },
            FieldRenderParams {
                block: b_notes,
                content: notes,
                area: r3,
                is_active: *curr_editing == Editing::Notes,
                textarea: self.textarea(Editing::Notes),
            },
        ];

        // 统一渲染逻辑
        for field in fields {
            let mut block = field.block;
            if field.is_active {
                block = block.fg(Color::Yellow);
                let inner = block.inner(field.area);
                block.render(field.area, buf);
                field.textarea.render(inner, buf);
            } else {
                Paragraph::new(field.content)
                    .block(block)
                    .render(field.area, buf);
            }
        }

    }
}
