use crate::app::tui::colors::{CL_D_WHITE, CL_L_BLACK, CL_WHITE};
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Color, Constraint, Modifier, StatefulWidget, Style, Stylize, Widget};
use ratatui::widgets::{Block, BorderType, List, ListItem, ListState, Padding};

/// 帮助页面实体
pub struct KeyMapInfo<'a> {
    pub key_map: &'a str,
    pub note: &'a str,
}

/// 帮助页面
pub struct HelpPage<'a, const N: usize> {
    pub key_maps: [KeyMapInfo<'a>; N],
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
        let tips_len = self.key_maps.len();
        let mut left_k = Vec::with_capacity(tips_len);
        let mut right_v = Vec::with_capacity(tips_len);
        for (i, KeyMapInfo {key_map, note}) in self.key_maps.iter().enumerate() {
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

impl HelpPage<'static, 9>{
    pub const fn editing() -> Self {
        Self {
            key_maps: [
                KeyMapInfo {
                    key_map: "<↓>",
                    note: "select next input-area | cursor move down",
                },
                KeyMapInfo {
                    key_map: "<↑>",
                    note: "select prev input-area | cursor move up",
                },
                KeyMapInfo {
                    key_map: "<←>",
                    note: "cursor move left",
                },
                KeyMapInfo {
                    key_map: "<→>",
                    note: "cursor move right",
                },
                KeyMapInfo {
                    key_map: "<ESC>",
                    note: "quit edit",
                },
                KeyMapInfo {
                    key_map: "<CTRL+C>",
                    note: "quit app",
                },
                KeyMapInfo {
                    key_map: "<ENTER>",
                    note: "select next input-area | new line",
                },
                KeyMapInfo {
                    key_map: "<TAB>",
                    note: "select next input-area",
                },
                KeyMapInfo {
                    key_map: "<CTRL+S>",
                    note: "save",
                },
            ]
        }
    }
}

impl HelpPage<'static, 3> {
    pub const fn detail() -> Self {
        Self {
            key_maps: [
                KeyMapInfo {
                    key_map: "<ESC>|<Q>",
                    note: "quit-detail",
                },
                KeyMapInfo {
                    key_map: "<D>",
                    note: "delete",
                },
                KeyMapInfo {
                    key_map: "<L>",
                    note: "quit-detail and re-lock",
                },
            ]
        }

    }

}

impl HelpPage<'static, 13> {
    pub const fn home_page() -> Self {
        Self {
            key_maps: [
                KeyMapInfo {
                    key_map: "<F>",
                    note: "find",
                },
                KeyMapInfo {
                    key_map: "<↓>|<J>",
                    note: "down",
                },
                KeyMapInfo {
                    key_map: "<↑>|<K>",
                    note: "up",
                },
                KeyMapInfo {
                    key_map: "<g>",
                    note: "first",
                },
                KeyMapInfo {
                    key_map: "<G>",
                    note: "last",
                },
                KeyMapInfo {
                    key_map: "<CTRL+C>|<Q>",
                    note: "quit app",
                },
                KeyMapInfo {
                    key_map: "<ESC>",
                    note: "quit app | [find] find | quit find",
                },
                KeyMapInfo {
                    key_map: "<ENTER>",
                    note: "detail current | [find] find",
                },
                KeyMapInfo {
                    key_map: "<O>",
                    note: "detail current",
                },
                KeyMapInfo {
                    key_map: "<I>",
                    note: "create new",
                },
                KeyMapInfo {
                    key_map: "<E>",
                    note: "edit current",
                },
                KeyMapInfo {
                    key_map: "<D>",
                    note: "delete current",
                },
                KeyMapInfo {
                    key_map: "<L>",
                    note: "re-lock",
                },
            ],
        }
    }
}
