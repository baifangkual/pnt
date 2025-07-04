use crate::app::consts::MAIN_PASS_MAX_RE_TRY;
use crate::app::entry::Entry;
use crate::app::tui::error::TError::ReTryMaxExceed;
use crate::app::tui::screen::Screen;
use anyhow::{Error, anyhow};
use ratatui::widgets::ListState;
use crate::app::encrypt::MainPwdVerifier;

/// 表示正在编辑 UserInputEntry的 哪一个
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum Editing {
    #[default]
    Name,
    Description,
    Identity,
    Password,
}

/// 主页/仪表盘 的状态信息
#[derive(Debug, Clone)]
pub struct DashboardState {
    // 控制 find_input 的 标志位
    pub find_mode: bool,
    pub find_input: String,
    pub entries: Vec<Entry>,
    pub cursor: ListState, // 添加ListState来控制滚动
}

impl DashboardState {
    pub fn new(entries: Vec<Entry>) -> Self {
        let mut cursor = ListState::default();
        cursor.select(if entries.is_empty() { None } else { Some(0) });
        Self {
            find_mode: false,
            find_input: String::new(),
            entries,
            cursor,
        }
    }

    /// 更新光标坐标
    pub fn update_cursor(&mut self, index: Option<usize>) {
        self.cursor.select(index);
    }

    /// 光标指向的 元素 在 vec 的 index
    pub fn cursor_selected(&self) -> Option<usize> {
        self.cursor.selected()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }
}

/// 主密码输入界面状态
#[derive(Debug, Clone)] // todo 后续该应载荷 main pwd verifier，避免验证密码时的重新构建
pub struct NeedMainPwdState {
    pub mp_input: String,
    pub on_ok_to_screen: Box<Screen>,
    pub retry_count: u8,
}
impl NeedMainPwdState {
    pub fn new(on_ok_to_screen: Screen) -> Self {
        Self {
            mp_input: String::new(),
            on_ok_to_screen: Box::new(on_ok_to_screen), // 因为和screen的自引用嵌套问题，遂使用box指针
            retry_count: 0,
        }
    }
    
    pub fn mp_input(&self) -> &str {
        &self.mp_input
    }

    pub fn retry_count(&self) -> u8 {
        self.retry_count
    }

    /// 尝试自增重试次数，若重试次数到顶 ([`MAIN_PASS_MAX_RE_TRY`])
    /// 则返回 Err
    pub fn increment_retry_count(&mut self) -> anyhow::Result<()> {
        if self.retry_count >= MAIN_PASS_MAX_RE_TRY {
            Err(Error::from(ReTryMaxExceed(self.retry_count)))
        } else {
            self.retry_count += 1;
            self.mp_input.clear();
            Ok(())
        }
    }
}
