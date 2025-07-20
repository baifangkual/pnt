use crate::app::tui::colors::{CL_DD_WHITE, CL_D_WHITE, CL_LL_BLACK, CL_WHITE};
use crate::app::tui::components::states::HomePageV1State;
use crate::app::tui::layout::RectExt;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Buffer, Color, Margin, Modifier, StatefulWidget, Style, Stylize, Text, Widget};
use ratatui::widgets::{
    Block, BorderType, Borders, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table,
};

pub struct HomePageV1Widget;

impl StatefulWidget for HomePageV1Widget {
    type State = HomePageV1State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [left, center, _] = Layout::horizontal([
            Constraint::Length(3), // 不能为 0，否则 滚动条组件报错Scrollbar area is empty，防止终端重大小调整
            Constraint::Percentage(96),
            Constraint::Length(1),
        ])
        .areas(area);

        let layout_v = Layout::vertical([
            Constraint::Length(3), // 搜索框
            Constraint::Fill(0),
        ]);

        // 搜索框， list 区域， 底部
        let [area_find, area_table] = layout_v.areas(center);

        // find 查找的字符渲染
        let mut find_input_block = Block::bordered().border_type(BorderType::Plain);
        let rect_query = area_find.h_centered_percent(80);
        // inner
        let rect_query_inner = find_input_block.inner(rect_query);
        // lr
        let [icon, query_line_rect] =
            Layout::horizontal([Constraint::Length(3), Constraint::Fill(0)]).areas(rect_query_inner);

        Paragraph::new("  ").fg(CL_D_WHITE).render(icon, buf);

        // 当前已输入的查找要求值
        let current_find_input = state.current_find_input();

        // find 时 框框 高亮
        if state.find_mode() {
            find_input_block = find_input_block.fg(Color::Yellow);
            find_input_block.render(rect_query, buf);
            state.render_text_area(query_line_rect, buf);
        } else {
            // 否则用 paragraph渲染，无光标
            find_input_block = find_input_block.fg(CL_D_WHITE);
            find_input_block.render(rect_query, buf);
            state.render_text_area(query_line_rect, buf);
        }

        let rows = state
            .entries()
            .iter()
            .filter(|&e| current_find_input.is_empty() || e.about.contains(current_find_input))
            .enumerate()
            .map(|(_, enc_entry)| {
                // fix 这里得用 clone，否则引用一直持续到调用 render，但是那里又需要可变引用，遂不行
                // 这里只能clone获取所有权，但是有string的clone开销，后续得想办法不用clone开销...
                let about = Text::from(enc_entry.about.clone());
                let notes = enc_entry.notes.as_ref().map(|s| s.to_owned()).unwrap_or("".to_owned());
                let notes = Text::from(notes);
                Row::new([about, notes]).fg(CL_WHITE)
            });

        let header_style = Style::default().fg(CL_WHITE).bg(CL_LL_BLACK);
        let header = Row::new(["About", "Notes"]).style(header_style);

        let table = Table::new(
            rows,
            [
                // 最小20，最大根据about长度计算之，最小20为防止about都短时过于靠左边...
                // + 1 is for padding.
                Constraint::Length(20.max(state.max_about_width() + 1)),
                Constraint::Fill(0),
            ],
        )
        .block(
            Block::new()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Plain)
                .fg(CL_DD_WHITE),
        )
        .header(header)
        .highlight_symbol(" ") // 填充，防止 about内容完全贴到左边
        .highlight_spacing(HighlightSpacing::Always)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED).fg(CL_DD_WHITE));

        StatefulWidget::render(table, area_table, buf, state.cursor_mut_ref());

        let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(CL_DD_WHITE))
            .symbols(ratatui::symbols::scrollbar::VERTICAL)
            .thumb_style(Style::default().fg(CL_DD_WHITE))
            .track_symbol(Some("|"))
            .begin_symbol(Some(ratatui::symbols::DOT))
            .end_symbol(Some(ratatui::symbols::DOT));


        let [_, area_table_left] = layout_v.areas(left);
        StatefulWidget::render(sb, area_table_left.inner(Margin::new(1, 0)), buf, state.scrollbar_state_mut());
    }
}
