use crate::app::tui::colors::{CL_L_BLACK, CL_LL_BLACK, CL_WHITE};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Stylize, Text, Widget};
use ratatui::widgets::{Block, BorderType, List, ListItem, Padding};

/// 帮助页面实体
pub struct HelpShowItem<'a> {
    pub key_map: &'a str,
    pub note: &'a str,
}

/// 帮助页面
pub struct HelpPage<'a, const N: usize> {
    pub tips: [HelpShowItem<'a>; N],
}

impl<'a, const N: usize> Widget for &HelpPage<'a, N> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("help")
            .fg(CL_WHITE)
            .border_type(BorderType::Plain)
            .padding(Padding::proportional(1));
        let inner_area = block.inner(area);
        block.render(area, buf);
        // Clear.render(inner_area, buf);

        // todo 调整显示，使用table......
        let li = self
            .tips
            .iter()
            .enumerate()
            .map(|(i, tip)| {
                let l = Line::raw(tip.key_map).fg(Color::Yellow).bold().left_aligned();
                let r = Line::raw(tip.note).fg(CL_WHITE).right_aligned();
                let text = Text::from(vec![l, r]);
                if i % 2 == 0 {
                    ListItem::new(text).bg(CL_L_BLACK)
                } else {
                    ListItem::new(text).bg(CL_LL_BLACK)
                }
            })
            .collect::<Vec<ListItem>>();
        List::new(li).render(inner_area, buf);
    }
}

impl HelpPage<'static, 15> {
    // todo 后续应修改为 不同页面不同 help 项
    pub const HELP_PAGE_DASHBOARD: HelpPage<'static, 15> = HelpPage::dashboard();
    const fn dashboard() -> Self {
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
