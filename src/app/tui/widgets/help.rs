use crate::app::tui::colors::{CL_L_BLACK, CL_LL_BLACK, CL_WHITE};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Stylize, Text, Widget};
use ratatui::widgets::{Block, BorderType, List, ListItem, Padding};
use std::sync::LazyLock;

/// 帮助页面实体
pub struct HelpShowItem {
    pub key_map: String,
    pub note: String,
}

/// 帮助页面
pub struct HelpPage {
    pub tips: Vec<HelpShowItem>,
}

/// 帮助页面单例, 即使没有多线程访问，rust也要求 static 为 Sync 的，所以使用 LazyLock
pub static HELP_PAGE_DASHBOARD: LazyLock<HelpPage> = LazyLock::new(|| HelpPage::dashboard());

// todo 后续应修改为 不同页面不同 help 项

impl Widget for &HelpPage {
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
                let l = Line::raw(&tip.key_map).fg(Color::Yellow).bold().left_aligned();
                let r = Line::raw(&tip.note).fg(CL_WHITE).right_aligned();
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

impl HelpPage {
    fn dashboard() -> Self {
        Self {
            tips: vec![
                HelpShowItem {
                    key_map: "<F1>".to_string(),
                    note: "help".to_string(),
                },
                HelpShowItem {
                    key_map: "f".to_string(),
                    note: "find".to_string(),
                },
                HelpShowItem {
                    key_map: "j | <DOWN>".to_string(),
                    note: "down".to_string(),
                },
                HelpShowItem {
                    key_map: "k | <UP>".to_string(),
                    note: "up".to_string(),
                },
                HelpShowItem {
                    key_map: "<Ctrl> c".to_string(),
                    note: "quit-app".to_string(),
                },
                HelpShowItem {
                    key_map: "q".to_string(),
                    note: "back screen | quit-app".to_string(),
                },
                HelpShowItem {
                    key_map: "<ESC>".to_string(),
                    note: "back screen | [edit] quit-edit | quit-app | [find] find".to_string(),
                },
                HelpShowItem {
                    key_map: "<Enter>".to_string(),
                    note: "detail current | [find] find".to_string(),
                },
                HelpShowItem {
                    key_map: "o".to_string(),
                    note: "detail current".to_string(),
                },
                HelpShowItem {
                    key_map: "i".to_string(),
                    note: "create new".to_string(),
                },
                HelpShowItem {
                    key_map: "e".to_string(),
                    note: "edit current".to_string(),
                },
                HelpShowItem {
                    key_map: "<Ctrl> s".to_string(),
                    note: "[edit] save edit".to_string(),
                },
                HelpShowItem {
                    key_map: "d".to_string(),
                    note: "delete current".to_string(),
                },
                HelpShowItem {
                    key_map: "<Tab>".to_string(),
                    note: "[edit] next textarea".to_string(),
                },
                HelpShowItem {
                    key_map: "l".to_string(),
                    note: "re-lock".to_string(),
                },
            ],
        }
    }
}
