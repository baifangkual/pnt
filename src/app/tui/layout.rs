use ratatui::layout::{Constraint, Layout, Rect};
use std::rc::Rc;


/// 固定大小-水平中央
#[inline]
pub fn h_centered_fixed(width: u16, rect: Rect) -> Rect {
    Layout::horizontal([Constraint::Fill(0), Constraint::Length(width), Constraint::Fill(0)]).areas::<3>(rect)[1]
}
/// 固定大小-垂直中央
#[inline]
pub fn v_centered_fixed(height: u16, rect: Rect) -> Rect {
    Layout::vertical([Constraint::Fill(0), Constraint::Length(height), Constraint::Fill(0)]).areas::<3>(rect)[1]
}
/// 固定大小-居中
#[inline]
pub fn centered_fixed(width: u16, height: u16, rect: Rect) -> Rect {
    h_centered_fixed(width, v_centered_fixed(height, rect))
}

/// 该方法将在给定的Rect上计算弹出窗口的Rect，x和y参数指定相对于给定的Rect的大小
///
/// 返回的弹出窗口一定位于中心，percent_x percent_y 用来说明需要占用的父窗口的大小
#[inline]
pub fn centered_percent(percent_width: u16, percent_height: u16, r: Rect) -> Rect {
    h_centered_percent(v_centered_percent(r, percent_height), percent_width)
}

/// 将 rect 分为左右两个等值 rect，
/// 方法返回 [rect; 2]
#[inline]
pub fn horizontal_split2(rect: Rect) -> [Rect; 2] {
    Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(rect)
}

/// 左右的居中 rect，参数centered_percent表示返回的占给定的rect的百分比
#[inline]
pub fn h_centered_percent(rect: Rect, centered_percent: u16) -> Rect {
    Layout::horizontal([
        Constraint::Percentage((100 - centered_percent) / 2),
        Constraint::Percentage(centered_percent),
        Constraint::Percentage((100 - centered_percent) / 2),
    ])
    .areas::<3>(rect)[1]
}

/// 上下的居中 rect，参数centered_percent表示返回的占给定的rect的百分比
#[inline]
pub fn v_centered_percent(rect: Rect, centered_percent: u16) -> Rect {
    Layout::vertical([
        Constraint::Percentage((100 - centered_percent) / 2),
        Constraint::Percentage(centered_percent),
        Constraint::Percentage((100 - centered_percent) / 2),
    ])
    .areas::<3>(rect)[1]
}

pub trait RectExt {
    fn h_centered_percent(self, centered_percent: u16) -> Self;
    fn v_centered_percent(self, centered_percent: u16) -> Self;
    fn centered_percent(self, percent_width: u16, percent_height: u16) -> Self;
}
impl RectExt for Rect {
    fn h_centered_percent(self, centered_percent: u16) -> Self {
        h_centered_percent(self, centered_percent)
    }
    fn v_centered_percent(self, centered_percent: u16) -> Self {
        v_centered_percent(self, centered_percent)
    }
    fn centered_percent(self, percent_width: u16, percent_height: u16) -> Self {
        centered_percent(percent_width, percent_height, self)
    }
}
