use crate::app::tui::screen::state::DashboardState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Buffer, Color, Line, Span, StatefulWidget, Style, Stylize, Text, Widget};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use crate::app::entry::EncryptedEntry;

/// 处理文本使其适应指定宽度，考虑中文字符
fn truncate_text(text: &str, max_width: usize) -> String {
    let mut width = 0;
    let mut result = String::new();

    for grapheme in text.graphemes(true) {
        let char_width = UnicodeWidthStr::width(grapheme);
        if width + char_width > max_width {
            if max_width > 3 {
                result.push_str("...");
            }
            break;
        }
        width += char_width;
        result.push_str(grapheme);
    }

    // 补充空格到指定宽度
    while width < max_width {
        result.push(' ');
        width += 1;
    }

    result
}

pub struct DashboardWidget;

const TITLE: &str = concat!(
    clap::crate_name!(),
    "-v",
    clap::crate_version!(),
    "-",
    "help:F1"
);

impl StatefulWidget for DashboardWidget {
    type State = DashboardState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered()
            .title(Line::from(TITLE))
            .fg(Color::White)
            .border_type(BorderType::Plain);

        // 创建内容区域
        let inner_area = block.inner(area);
        block.render(area, buf);
        // 布局 1| f | 1
        let layout_1f1 = Layout::default()
            .direction(Direction::Horizontal) // 水平
            .constraints([
                Constraint::Max(5),
                Constraint::Percentage(90),
                Constraint::Max(5),
            ])
            .split(inner_area);

        // 布局 上下 5 | 93 | 2
        let layout_hm1 = Layout::default()
            .direction(Direction::Vertical) // 垂直
            .constraints([
                Constraint::Min(3), // 搜索框
                Constraint::Percentage(95),
                Constraint::Min(2),
            ])
            .split(layout_1f1[1]);

        // 布局 左右 搜索框左右都减小
        let layout_l_find_r = Layout::default()
            .direction(Direction::Horizontal) // 垂直
            .constraints([
                Constraint::Percentage(10), // 搜索框
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(layout_hm1[0]);

        // find 查找的字符渲染
        let mut find_input_block = Block::bordered()
            .border_type(BorderType::Plain);

        // find 时 框框 高亮
        if state.find_mode {
            find_input_block = find_input_block.fg(Color::Yellow)
        } else {
            find_input_block = find_input_block.fg(Color::White)
        }
        // find_input_block.render(layout_l_find_r[1], buf);
        Paragraph::new(state.find_input.as_str())
            .block(find_input_block)
            .left_aligned()
            .render(layout_l_find_r[1], buf);

        let show_vec = if state.find_input.is_empty() {
        // 若未要查找，则所有显示
            state.entries().iter().collect::<Vec<_>>()
        } else {
            // 否则过滤查找的
            let ref_find = state.find_input.as_str();
            state.entries().iter()
                .filter(|e| e.name.contains(ref_find))
                .collect::<Vec<_>>()
        };

        // list 区域
        let inner_block = Block::new()
            .borders(Borders::BOTTOM | Borders::TOP)
            .border_type(BorderType::Plain)
            .fg(Color::White);

        // 计算每列的宽度比例（去除边框和padding后的可用宽度）
        let available_width = inner_area.width as usize - 4; // 减去左右边距
        let index_width = (available_width * 5) / 100; // 10% 用于索引
        let name_width = (available_width * 30) / 100; // 30% 用于名称
        let desc_width = available_width - index_width - name_width; // 剩余用于描述

        // 创建列表项
        let items: Vec<ListItem> = show_vec
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let index_str = format!("{:>width$}", i, width = index_width);
                let name_str = truncate_text(&entry.name, name_width);
                let desc_str =
                    truncate_text(entry.description.as_deref().unwrap_or(""), desc_width);
                let line_content = format!("{} | {} │ {}", index_str, name_str, desc_str);
                ListItem::new(Line::from(line_content))
            })
            .collect();

        // 创建列表并设置样式
        let list = List::new(items).block(inner_block).highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::from_u32(0xD9D9D9)),
        );

        // 使用 StatefulWidget 渲染
        StatefulWidget::render(list, layout_hm1[1], buf, &mut state.cursor);
    }
}
