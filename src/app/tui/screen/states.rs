use crate::app::consts::MAIN_PASS_MAX_RE_TRY;
use crate::app::crypto::Encrypter;
use crate::app::entry::{EncryptedEntry, InputEntry, ValidEntry};
use crate::app::errors::AppError::ReTryMaxExceed;
use crate::app::tui::screen::Screen;
use crate::app::tui::widgets::InputField;
use anyhow::{Context, anyhow};
use ratatui::widgets::ListState;
use crate::app::tui::intents::EnterScreenIntent;

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
            Editing::Name => &mut self.u_input.name,
            Editing::Description => &mut self.u_input.description,
            Editing::Identity => &mut self.u_input.identity,
            Editing::Password => &mut self.u_input.password,
        }
    }

    pub fn new_updating(u_input: InputEntry, e_id: u32) -> Self {
        Self {
            editing: Editing::Name,
            u_input,
            e_id: Some(e_id),
        }
    }

    pub fn new_creating() -> Self {
        Self {
            editing: Editing::Name,
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
            Editing::Name => Editing::Description,
            Editing::Identity => Editing::Name,
            Editing::Password => Editing::Identity,
            Editing::Description => Editing::Password,
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
            Editing::Name => Editing::Identity,
            Editing::Identity => Editing::Password,
            Editing::Password => Editing::Description,
            Editing::Description => Editing::Name,
        }
    }
}

/// 表示正在编辑 UserInputEntry的 哪一个
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum Editing {
    #[default]
    Name,
    Identity,
    Password,
    Description,
}

/// 主页/仪表盘 的状态信息
#[derive(Debug, Clone)]
pub struct DashboardState {
    // 控制 find_input 的 标志位
    pub find_mode: bool,
    pub find_input: String,
    pub entries: Vec<EncryptedEntry>,
    pub cursor: ListState, // 添加ListState来控制滚动
}

impl DashboardState {
    pub fn new(entries: Vec<EncryptedEntry>) -> Self {
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

    pub fn entries(&self) -> &Vec<EncryptedEntry> {
        &self.entries
    }
}

/// 主密码输入界面状态
#[derive(Debug, Clone)] // todo 后续该应载荷 main pwd verifier，避免验证密码时的重新构建
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

    /// 尝试自增重试次数，若重试次数到顶 ([`MAIN_PASS_MAX_RE_TRY`])
    /// 则返回 Err
    pub fn increment_retry_count(&mut self) -> anyhow::Result<()> {
        if self.retry_count >= MAIN_PASS_MAX_RE_TRY {
            Err(ReTryMaxExceed)?
        } else {
            self.retry_count += 1;
            self.mp_input.clear();
            Ok(())
        }
    }
}
