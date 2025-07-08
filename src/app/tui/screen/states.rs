use crate::app::consts::ALLOC_VALID_MAIN_PASS_MAX;
use crate::app::crypto::Encrypter;
use crate::app::entry::{EncryptedEntry, InputEntry, ValidEntry};
use crate::app::errors::AppError::ValidPassword;
use crate::app::tui::intents::EnterScreenIntent;
use anyhow::{anyhow, Context};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Style, Widget};
use ratatui::widgets::{ListState, ScrollbarState};
use tui_textarea::TextArea;

#[derive(Debug, Clone)]
pub struct EditingState {
    editing: Editing,
    u_input: InputEntry,
    /// 正在编辑的条目id，若为None，则表示正在编辑的条目为新建条目
    e_id: Option<u32>,
}

impl Default for EditingState {
    fn default() -> Self {
        EditingState::new_creating()
    }
}

impl EditingState {
    pub fn current_input_entry(&self) -> &InputEntry {
        &self.u_input
    }

    /// 返回当前正在编辑的字段是哪一个
    pub fn current_editing_type(&self) -> &Editing {
        &self.editing
    }

    /// 返回当前正在编辑的字段的可变引用
    pub fn current_editing_string_mut(&mut self) -> &mut String {
        match self.editing {
            Editing::About => &mut self.u_input.about,
            Editing::Notes => &mut self.u_input.notes,
            Editing::Username => &mut self.u_input.username,
            Editing::Password => &mut self.u_input.password,
        }
    }

    pub fn new_updating(u_input: InputEntry, e_id: u32) -> Self {
        Self {
            editing: Editing::About,
            u_input,
            e_id: Some(e_id),
        }
    }

    pub fn new_creating() -> Self {
        Self {
            editing: Editing::About,
            u_input: InputEntry::default(),
            e_id: None,
        }
    }

    pub fn current_e_id(&self) -> Option<u32> {
        self.e_id
    }

    /// 光标向上移动，若当前光标为Name，则移动到Password
    pub fn cursor_up(&mut self) {
        self.editing = match self.editing {
            Editing::About => Editing::Notes,
            Editing::Username => Editing::About,
            Editing::Password => Editing::Username,
            Editing::Notes => Editing::Password,
        }
    }

    pub fn current_input_validate(&self) -> bool {
        self.u_input.validate()
    }

    /// 尝试加密 UserInputEntry 为 ValidInsertEntry
    /// 当 UserInputEntry 不合法时，该方法会返回错误
    /// 当 UserInputEntry 合法时, 该方法会返回 ValidInsertEntry 和 可能的 条目id
    /// 当条目id为None时，表示该条目为新建条目, 反之则为更新条目
    pub fn try_encrypt<'a, Enc>(
        &'a self,
        encrypter: &Enc,
    ) -> anyhow::Result<ValidEntry>
    where
        Enc: Encrypter<&'a InputEntry, ValidEntry>,
    {
        if !self.current_input_validate() {
            return Err(anyhow!("input not validate"));
        }
        Ok(encrypter.encrypt(&self.u_input)?)
    }

    /// 光标向下移动，若当前光标为Password，则移动到Name
    pub fn cursor_down(&mut self) {
        self.editing = match self.editing {
            Editing::About => Editing::Username,
            Editing::Username => Editing::Password,
            Editing::Password => Editing::Notes,
            Editing::Notes => Editing::About,
        }
    }
}

/// 表示正在编辑 UserInputEntry的 哪一个
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum Editing {
    #[default]
    About,
    Username,
    Password,
    Notes,
}

/// 主页/仪表盘 的状态信息
#[derive(Debug, Clone)]
pub struct DashboardState {
    // 控制 find_input 的 标志位
    pub find_mode: bool,
    find_input: TextArea<'static>,
    entries: Vec<EncryptedEntry>,
    cursor: ListState, // 添加ListState来控制滚动
    scrollbar_state: ScrollbarState // 垂直滚动条样式
}

impl DashboardState {

    /// 根据给定的 entries 创建
    ///
    /// 该方法内会
    pub fn new(mut entries: Vec<EncryptedEntry>) -> Self {
        Self::sort_entries(&mut entries);
        let mut cursor = ListState::default();
        cursor.select(if entries.is_empty() { None } else { Some(0) });
        let scrollbar_state = ScrollbarState::new(entries.len());
        Self {
            find_mode: false,
            find_input: Self::init_input_textarea(),
            entries,
            cursor,
            scrollbar_state
        }
    }

    fn init_input_textarea() -> TextArea<'static> {
        let mut textarea = TextArea::default();
        textarea.set_placeholder_text("find");
        textarea.set_cursor_line_style(Style::default());
        textarea
    }

    pub fn current_find_input(&self) -> &str {
        self.find_input.lines().last().unwrap()
    }

    // 妈的生命周期传染
    pub fn find_textarea(& mut self) -> &mut TextArea<'static> {
        &mut self.find_input
    }

    pub fn render_text_area(&self, area: Rect, buf: &mut Buffer){
        self.find_input.render(area, buf);
    }

    pub fn clear_find_input(&mut self) {
        self.find_input = Self::init_input_textarea();
    }

    /// 对 entries 进行排序
    fn sort_entries(entries:&mut Vec<EncryptedEntry>) {
        entries.sort_unstable_by(EncryptedEntry::sort_by_update_time);
    }

    /// 更新光标坐标
    fn update_cursor(&mut self, index: Option<usize>) {
        self.cursor.select(index);
    }

    /// 光标指向的 元素 在 vec 的 index
    pub fn cursor_selected(&self) -> Option<usize> {
        self.cursor.selected()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn cursor_mut_ref(&mut self) -> &mut ListState {
        &mut self.cursor
    }

    pub fn entries(&self) -> &Vec<EncryptedEntry> {
        &self.entries
    }

    pub fn cursor_down(&mut self) {
        if let Some(p) = self.cursor_selected() {
            if p >= self.entry_count() - 1 {
                self.update_cursor(Some(0))
            } else {
                self.cursor_mut_ref().select_next();
            }
        }
    }

    pub fn cursor_up(&mut self) {
        if let Some(p) = self.cursor_selected() {
            if p == 0 {
                self.update_cursor(Some(self.entry_count() - 1))
            } else {
                self.cursor_mut_ref().select_previous();
            }
        }
    }

    /// 获取滚动条可变引用
    ///
    /// 该方法内会进行滚动条的状态更新
    pub fn scrollbar_state_mut(&mut self) -> &mut ScrollbarState {
        if let Some(c_ptr) = self.cursor.selected() {
            self.scrollbar_state = self.scrollbar_state.position(c_ptr);
        }
        &mut self.scrollbar_state
    }

    /// 重置载荷的 entries，
    /// 该方法内会进行 entries 的 sort
    ///
    /// 该方法内也会同步更新 cursor and scrollbar 状态
    pub fn reset_entries(&mut self, mut entries: Vec<EncryptedEntry>) {
        Self::sort_entries(&mut entries);
        self.scrollbar_state = self.scrollbar_state.content_length(entries.len());
        self.entries = entries;
        if !self.entries().is_empty() {
            if let None = self.cursor_selected() {
                self.update_cursor(Some(0))
            }
        } else {
            self.update_cursor(None);
        }
    }
}

/// 主密码输入界面状态
#[derive(Debug)]
pub struct NeedMainPwdState {
    pub mp_input: String,
    pub enter_screen_intent: Option<EnterScreenIntent>, // 一定有，应去掉该Option包装，但是 hold_mp_verifier_and_enter_target_screen 会无法通过编译
    pub retry_count: u8,
}
impl NeedMainPwdState {
    pub fn new(enter_screen_intent: EnterScreenIntent) -> Self {
        Self {
            mp_input: String::new(),
            enter_screen_intent: Some(enter_screen_intent),
            retry_count: 0,
        }
    }

    pub fn take_target_screen(&mut self) -> anyhow::Result<EnterScreenIntent> {
        self.enter_screen_intent
            .take()
            .context("'NeedMainPwdState' not found target screen")
    }

    pub fn mp_input(&self) -> &str {
        &self.mp_input
    }

    pub fn retry_count(&self) -> u8 {
        self.retry_count
    }

    /// 尝试自增重试次数，若重试次数到顶 ([`ALLOC_VALID_MAIN_PASS_MAX`])
    /// 则返回 Err
    pub fn increment_retry_count(&mut self) -> anyhow::Result<()> {
        if self.retry_count + 1 >= ALLOC_VALID_MAIN_PASS_MAX {
            Err(ValidPassword)?
        } else {
            self.retry_count += 1;
            self.mp_input.clear();
            Ok(())
        }
    }
}
