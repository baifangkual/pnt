use ratatui::layout::{Constraint, Layout, Rect};
use std::rc::Rc;

/// 该方法将在给定的Rect上计算弹出窗口的Rect，x和y参数指定相对于给定的Rect的大小
///
/// 返回的弹出窗口一定位于中心，percent_x percent_y 用来说明需要占用的父窗口的大小
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    h_centered_rect(v_centered_rect(r, percent_y), percent_x)
}

/// 将 rect 分为左右两个等值 rect，
/// 方法返回 [rect; 2]
pub fn split_lr_rects(rect: Rect) -> Rc<[Rect]> {
    Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rect)
}

/// 左右的居中 rect，参数centered_percent表示返回的占给定的rect的百分比
pub fn h_centered_rect(rect: Rect, centered_percent: u16) -> Rect {
    Layout::horizontal([
        Constraint::Percentage((100 - centered_percent) / 2),
        Constraint::Percentage(centered_percent),
        Constraint::Percentage((100 - centered_percent) / 2),
    ])
    .split(rect)[1]
}

/// 上下的居中 rect，参数centered_percent表示返回的占给定的rect的百分比
pub fn v_centered_rect(rect: Rect, centered_percent: u16) -> Rect {
    Layout::vertical([
        Constraint::Percentage((100 - centered_percent) / 2),
        Constraint::Percentage(centered_percent),
        Constraint::Percentage((100 - centered_percent) / 2),
    ])
    .split(rect)[1]
}

pub trait RectExt {
    fn h_centered_rect(self, centered_percent: u16) -> Self;
    fn v_centered_rect(self, centered_percent: u16) -> Self;
    fn centered_rect(self, percent_x: u16, percent_y: u16) -> Self;
}
impl RectExt for Rect {
    fn h_centered_rect(self, centered_percent: u16) -> Self {
        h_centered_rect(self, centered_percent)
    }
    fn v_centered_rect(self, centered_percent: u16) -> Self {
        v_centered_rect(self, centered_percent)
    }
    fn centered_rect(self, percent_x: u16, percent_y: u16) -> Self {
        centered_rect(percent_x, percent_y, self)
    }
}
