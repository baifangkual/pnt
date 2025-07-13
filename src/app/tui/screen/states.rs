use crate::app::consts::ALLOC_INVALID_MAIN_PASS_MAX;
use crate::app::crypto::Encrypter;
use crate::app::entry::{EncryptedEntry, InputEntry, ValidEntry};
use crate::app::errors::AppError::InvalidPassword;
use crate::app::tui::intents::EnterScreenIntent;
use crate::app::tui::widgets::{TextAreaExt, new_input_textarea};
use anyhow::{Context, anyhow};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::widgets::{ListState, ScrollbarState};
use tui_textarea::TextArea;

#[derive(Debug, Clone)]
pub struct EditingState {
    editing: Editing,
    input_textarea: [TextArea<'static>; 4],
    /// 正在编辑的条目id，若为None，则表示正在编辑的条目为新建条目
    e_id: Option<u32>,
}

impl Default for EditingState {
    fn default() -> Self {
        EditingState::new_creating()
    }
}

impl EditingState {
    pub fn current_input_entry(&self) -> InputEntry {
        InputEntry {
            about: self.value(Editing::About),
            username: self.value(Editing::Username),
            password: self.value(Editing::Password),
            notes: self.value(Editing::Notes),
        }
    }

    /// 返回当前正在编辑的字段是哪一个
    pub fn current_editing_type(&self) -> Editing {
        self.editing
    }

    /// 返回当前正在编辑的字段的可变引用
    pub fn current_editing_string_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.input_textarea[self.editing]
    }

    pub fn new_updating(u_input: InputEntry, e_id: u32) -> Self {
        let mut new = Self::new_creating();
        new.input_textarea[0].insert_str(u_input.about);
        new.input_textarea[1].insert_str(u_input.username);
        new.input_textarea[2].insert_str(u_input.password);
        new.input_textarea[3].insert_str(u_input.notes);
        new.e_id = Some(e_id);
        new
    }

    pub fn new_creating() -> Self {
        let editing = Editing::default();
        let mut textarea4 = Self::new4();
        textarea4[editing].set_activate_state(true); // 光标可见
        Self {
            editing,
            input_textarea: textarea4,
            e_id: None,
        }
    }

    /// 输入框4个
    fn new4() -> [TextArea<'static>; 4] {
        [
            new_input_textarea(Some("require about"), false),
            new_input_textarea(Some("require username"), false),
            new_input_textarea(Some("require password"), false),
            new_input_textarea(None, false),
        ]
    }

    /// 返回指定的输入框
    pub fn textarea(&self, editing: Editing) -> &TextArea<'static> {
        &self.input_textarea[editing]
    }
    /// 返回持有的所有textarea切片引用
    pub fn all_textarea(&self) -> &[TextArea<'static>; 4] {
        &self.input_textarea
    }

    /// 某个框的内容
    fn value(&self, editing: Editing) -> String {
        match editing {
            Editing::Notes => self.input_textarea[editing].lines().join("\n"),
            // textarea的lines.last().unwrap一定不会panic，因为其即使空字符串一定有值""...
            _ => self.input_textarea[editing].lines().last().unwrap().to_owned(),
        }
    }

    pub fn current_e_id(&self) -> Option<u32> {
        self.e_id
    }

    /// 光标向上移动，若当前光标为Name，则移动到Password
    pub fn cursor_up(&mut self) {
        // 将当前置位光标隐藏，将新的置位光标不隐藏
        self.input_textarea[self.editing].set_activate_state(false);
        self.editing = match self.editing {
            Editing::About => Editing::Notes,
            Editing::Username => Editing::About,
            Editing::Password => Editing::Username,
            Editing::Notes => Editing::Password,
        };
        self.input_textarea[self.editing].set_activate_state(true);
    }

    /// 光标向下移动，若当前光标为Password，则移动到Name
    pub fn cursor_down(&mut self) {
        self.input_textarea[self.editing].set_activate_state(false);
        self.editing = match self.editing {
            Editing::About => Editing::Username,
            Editing::Username => Editing::Password,
            Editing::Password => Editing::Notes,
            Editing::Notes => Editing::About,
        };
        self.input_textarea[self.editing].set_activate_state(true);
    }

    /// 当前输入是否有效
    ///
    /// 有效要求：
    /// * about 不为空
    /// * username 不为空
    /// * password 不为空
    ///
    /// # Panics
    /// 当 about username password中任意一个有多行内容时
    pub fn current_input_validate(&self) -> bool {
        // 0,1,2 notes 不校验
        for idx in 0..3usize {
            let text_area = &self.input_textarea[idx];
            // 不得为空
            if text_area.is_empty() {
                return false;
            }
            // 不得多行（或者说有换行符）(饱和校验）
            if text_area.lines().len() > 1 {
                // 该情况饱和的验证，用以校验设计上漏洞，正常用户输入因前置的按键拦截，
                // 其一定不为多行
                panic!("Invalid input");
            }
        }
        true
    }

    /// 尝试加密 UserInputEntry 为 ValidInsertEntry
    /// 当 UserInputEntry 不合法时，该方法会返回错误
    /// 当 UserInputEntry 合法时, 该方法会返回 ValidInsertEntry 和 可能的 条目id
    /// 当条目id为None时，表示该条目为新建条目, 反之则为更新条目
    pub fn try_encrypt<Enc>(&self, encrypter: &Enc) -> anyhow::Result<ValidEntry>
    where
        Enc: for<'a> Encrypter<&'a InputEntry, ValidEntry>,
    {
        if !self.current_input_validate() {
            return Err(anyhow!("input not validate"));
        }
        Ok(encrypter.encrypt(&self.current_input_entry())?)
    }
}
/// 表示正在编辑 UserInputEntry的 哪一个
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Editing {
    #[default]
    About = 0_u8,
    Username = 1_u8,
    Password = 2_u8,
    Notes = 3_u8,
}

impl<T> std::ops::Index<Editing> for [T; 4] {
    type Output = T;
    fn index(&self, editing: Editing) -> &T {
        &self[editing as usize]
    }
}
impl<T> std::ops::IndexMut<Editing> for [T; 4] {
    fn index_mut(&mut self, editing: Editing) -> &mut T {
        &mut self[editing as usize]
    }
}

/// 主页/仪表盘 的状态信息
#[derive(Debug, Clone)]
pub struct DashboardState {
    // 控制 find_input 的 标志位
    find_mode: bool,
    find_input: TextArea<'static>,
    entries: Vec<EncryptedEntry>,
    cursor: ListState,               // 添加ListState来控制滚动
    scrollbar_state: ScrollbarState, // 垂直滚动条样式
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
            find_input: new_input_textarea(Some("find"), false),
            entries,
            cursor,
            scrollbar_state,
        }
    }

    pub fn set_find_mode(&mut self, mode: bool) {
        self.find_mode = mode;
        self.find_input.set_activate_state(mode)
    }
    pub fn find_mode(&self) -> bool {
        self.find_mode
    }

    pub fn current_find_input(&self) -> &str {
        self.find_input.lines().last().unwrap()
    }

    // 妈的生命周期传染
    pub fn find_textarea(&mut self) -> &mut TextArea<'static> {
        &mut self.find_input
    }

    pub fn render_text_area(&self, area: Rect, buf: &mut Buffer) {
        self.find_input.render(area, buf);
    }

    pub fn clear_find_input(&mut self) {
        self.find_input = new_input_textarea(Some("find"), false);
    }

    /// 对 entries 进行排序
    fn sort_entries(entries: &mut [EncryptedEntry]) {
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
            if self.cursor_selected().is_none() {
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

    /// 尝试自增重试次数，若重试次数到顶 ([`ALLOC_INVALID_MAIN_PASS_MAX`])
    /// 则返回 Err
    pub fn increment_retry_count(&mut self) -> anyhow::Result<()> {
        if self.retry_count + 1 >= ALLOC_INVALID_MAIN_PASS_MAX {
            Err(InvalidPassword)?
        } else {
            self.retry_count += 1;
            self.mp_input.clear();
            Ok(())
        }
    }
}
