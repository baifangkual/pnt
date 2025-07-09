use crate::app::tui::layout::RectExt;
use crate::app::tui::screen::states::DashboardState;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Buffer, Color, Line, Margin, StatefulWidget, Style, Stylize, Widget};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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

impl StatefulWidget for DashboardWidget {
    type State = DashboardState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [l_5, area90_center, _] =
            Layout::horizontal([Constraint::Fill(5), Constraint::Percentage(80), Constraint::Fill(5)]).areas(area);

        let layout_v = Layout::vertical([
            Constraint::Min(3), // 搜索框
            Constraint::Percentage(100),
            Constraint::Min(1),
        ]);

        // 搜索框， list 区域， 底部
        let [q, l, b] = layout_v.areas(area90_center);

        // find 查找的字符渲染
        let mut find_input_block = Block::bordered().border_type(BorderType::Plain);
        let rect_query = q.h_centered_percent(80);
        // inner
        let rect_query_inner = find_input_block.inner(rect_query);
        // lr
        let [icon, query_line_rect] =
            Layout::horizontal([Constraint::Length(3), Constraint::Fill(0)]).areas(rect_query_inner);
        Paragraph::new("  ").fg(Color::from_u32(0xC6C8CC)).render(icon, buf);

        // 当前已输入的查找要求值
        let current_find_input = state.current_find_input();

        // find 时 框框 高亮
        if state.find_mode {
            find_input_block = find_input_block.fg(Color::Yellow);
            find_input_block.render(rect_query, buf);
            // 使用 textArea组件的渲染，渲染带光标
            state.render_text_area(query_line_rect, buf);
        } else {
            // 否则用 paragraph渲染，无光标
            find_input_block = find_input_block.fg(Color::from_u32(0xC6C8CC));
            find_input_block.render(rect_query, buf);

            // find_input 是否无值，无值则出现placeholder
            if current_find_input.is_empty() {
                Paragraph::new(" find")
                    .fg(Color::DarkGray)
                    .left_aligned()
                    .render(query_line_rect, buf);
            } else {
                Paragraph::new(current_find_input)
                    .left_aligned()
                    .render(query_line_rect, buf);
            }
        }

        // list 区域
        let inner_block = Block::new()
            .borders(Borders::BOTTOM | Borders::TOP)
            .border_type(BorderType::Plain)
            .fg(Color::from_u32(0x6E737C));

        // 计算每列的宽度比例（去除边框和padding后的可用宽度）
        let available_width = area.width as usize - 5; // 减去左右边距
        let index_width = (available_width * 5) / 100; // 5% 用于索引
        let name_width = (available_width * 30) / 100; // 30% 用于名称
        let desc_width = available_width - index_width - name_width; // 剩余用于描述

        // 创建列表项
        let items: Vec<ListItem> = state
            .entries()
            .iter() // 编译器和处理器会优化下 filter is_empty 判定...
            .filter(|&e| current_find_input.is_empty() || e.about.contains(current_find_input))
            .enumerate()
            .map(|(i, entry)| {
                let index_str = format!("{:>width$}", i, width = index_width);
                let name_str = truncate_text(&entry.about, name_width);
                let desc_str = truncate_text(entry.notes.as_deref().unwrap_or(""), desc_width);
                // let name_str = &entry.about;
                // let desc_str = entry.notes.as_deref().unwrap_or("_");
                let line_content = format!("{} | {} │ {}", index_str, name_str, desc_str);
                ListItem::new(Line::from(line_content))
            })
            .collect();

        // 创建列表并设置样式
        let list = List::new(items)
            .block(inner_block)
            .fg(Color::from_u32(0xDADBDE))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::from_u32(0xD9D9D9)));
        // .highlight_symbol(&e_id);

        // 使用 StatefulWidget 渲染
        StatefulWidget::render(list, l, buf, state.cursor_mut_ref());

        let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::from_u32(0x6E737C)))
            .symbols(ratatui::symbols::scrollbar::VERTICAL)
            .thumb_style(Style::default().fg(Color::from_u32(0xC6C8CC)))
            .track_symbol(Some("|"))
            .begin_symbol(Some(ratatui::symbols::DOT))
            .end_symbol(Some(ratatui::symbols::DOT));
        // 使用左边...
        let [_, l_m, _] = layout_v.areas(l_5);

        StatefulWidget::render(sb, l_m.inner(Margin::new(1, 0)), buf, state.scrollbar_state_mut());
    }
}
