use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Buffer, Color, Line, Span, StatefulWidget, Style, Stylize};
use ratatui::widgets::{Block, BorderType, List, ListItem};
use crate::app::tui::screen::DashboardState;

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

        // 计算每列的宽度比例（去除边框和padding后的可用宽度）
        let available_width = inner_area.width as usize - 4; // 减去左右边距
        let index_width = (available_width * 10) / 100;  // 10% 用于索引
        let name_width = (available_width * 30) / 100;   // 30% 用于名称
        let desc_width = available_width - index_width - name_width; // 剩余用于描述

        // 创建列表项
        let items: Vec<ListItem> = state.entries.iter().enumerate()
            .map(|(i, entry)| {
                let index_str = format!("{:>width$}", i, width = index_width);
                let name_str = truncate_text(&entry.name, name_width);
                let desc_str = truncate_text(
                    entry.description.as_deref().unwrap_or(""),
                    desc_width
                );

                let line_content = format!("{} │ {} │ {}", index_str, name_str, desc_str);

                if let Some(i) = state.cursor_selected() {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            line_content,
                            Style::default().fg(Color::Black).bg(Color::White),
                        )
                    ]))
                } else {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            line_content,
                            Style::default().fg(Color::White),
                        )
                    ]))
                }
            })
            .collect();

        // 创建列表并设置样式
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
            )
            .highlight_symbol(">> ");

        // 使用 StatefulWidget 渲染
        StatefulWidget::render(list, area, buf, &mut state.cursor);
    }
}