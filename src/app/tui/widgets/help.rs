use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Stylize, Widget};
use ratatui::widgets::{Block, BorderType, Clear, List, ListItem};
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
            .fg(Color::White)
            .border_type(BorderType::Plain);
        let inner_area = block.inner(area);
        block.render(area, buf);
        Clear.render(inner_area, buf);

        let li = self
            .tips
            .iter()
            .map(|tip| format!("{:>10}          {:<20}", tip.key_map, tip.note))
            .map(|tl| ListItem::new(tl).fg(Color::White))
            .collect::<Vec<ListItem>>();
        let rect = super::super::layout::centered_rect(90, 90, inner_area);
        List::new(li).render(rect, buf);
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
                    note: "back screen | quit-edit | quit-app | quit-find".to_string(),
                },
                HelpShowItem {
                    key_map: "<Enter>".to_string(),
                    note: "detail current | quit-find".to_string(),
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
                    key_map: "u".to_string(),
                    note: "update current".to_string(),
                },
                HelpShowItem {
                    key_map: "<Ctrl> s".to_string(),
                    note: "save edit".to_string(),
                },
                HelpShowItem {
                    key_map: "d".to_string(),
                    note: "delete current".to_string(),
                },
            ],
        }
    }
}
