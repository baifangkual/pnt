use crate::app::tui::colors::{CL_D_WHITE, CL_L_BLACK, CL_WHITE};
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Color, Constraint, Modifier, StatefulWidget, Style, Stylize, Widget};
use ratatui::widgets::{Block, BorderType, List, ListItem, ListState, Padding};

/// 帮助页面实体
pub struct HelpShowItem<'a> {
    pub key_map: &'a str,
    pub note: &'a str,
}

/// 帮助页面
pub struct HelpPage<'a, const N: usize> {
    pub tips: [HelpShowItem<'a>; N],
}

impl<'a, const N: usize> StatefulWidget for &HelpPage<'a, N> {
    type State = ListState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {

        let block = Block::bordered()
            .title("help")
            .fg(CL_WHITE)
            .border_type(BorderType::Plain)
            .padding(Padding::proportional(1));
            // .border_set(ratatui::symbols::border::EMPTY);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let [ l, r] = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ]).areas(inner_area);

        // to do 光标二分
        let tips_len = self.tips.len();
        let mut left_k = Vec::with_capacity(tips_len);
        let mut right_v = Vec::with_capacity(tips_len);
        for (i, HelpShowItem {key_map, note}) in self.tips.iter().enumerate() {
            let k = ListItem::new(*key_map).fg(Color::Yellow).bold();
            let v = ListItem::new(*note).fg(CL_D_WHITE);
            left_k.push(k);
            right_v.push(v);
        }

        const SELECTED_STYLE: Style = Style::new().bg(CL_L_BLACK).add_modifier(Modifier::BOLD);

        let l_list = List::new(left_k)
            .highlight_style(SELECTED_STYLE);
        let r_list = List::new(right_v)
            .highlight_style(SELECTED_STYLE);

        StatefulWidget::render(l_list, l, buf, state);
        StatefulWidget::render(r_list, r, buf, state);
    }
}

    // todo 后续应修改为 不同页面不同 help 项
impl HelpPage<'static, 17> {
    pub const HELP_HOME_PAGE: HelpPage<'static, 17> = HelpPage::home_page();
    const fn home_page() -> Self {
        Self {
            tips: [
                HelpShowItem {
                    key_map: "<F1>",
                    note: "help",
                },
                HelpShowItem {
                    key_map: "f",
                    note: "find",
                },
                HelpShowItem {
                    key_map: "j | <DOWN>",
                    note: "down",
                },
                HelpShowItem {
                    key_map: "k | <UP>",
                    note: "up",
                },
                HelpShowItem {
                    key_map: "g",
                    note: "first",
                },
                HelpShowItem {
                    key_map: "G",
                    note: "last",
                },
                HelpShowItem {
                    key_map: "<Ctrl> c",
                    note: "quit-app",
                },
                HelpShowItem {
                    key_map: "q",
                    note: "back screen | quit-app",
                },
                HelpShowItem {
                    key_map: "<ESC>",
                    note: "back screen | [edit] quit-edit | quit-app | [find] find",
                },
                HelpShowItem {
                    key_map: "<Enter>",
                    note: "detail current | [find] find",
                },
                HelpShowItem {
                    key_map: "o",
                    note: "detail current",
                },
                HelpShowItem {
                    key_map: "i",
                    note: "create new",
                },
                HelpShowItem {
                    key_map: "e",
                    note: "edit current",
                },
                HelpShowItem {
                    key_map: "<Ctrl> s",
                    note: "[edit] save edit",
                },
                HelpShowItem {
                    key_map: "d",
                    note: "delete current",
                },
                HelpShowItem {
                    key_map: "<Tab>",
                    note: "[edit] next textarea",
                },
                HelpShowItem {
                    key_map: "l",
                    note: "re-lock",
                },
            ],
        }
    }
}
