use crate::app::entry::{DetailEntry, Entry, UserInputEntry, ValidInsertEntry};

/// 帮助页面实体
pub struct  HelpEntry{
    pub key_map: String,
    pub note: String,
}

/// 当前屏幕
#[derive(Debug, Clone)] // todo 后续应移除重的 Clone
pub enum Screen {
    /// 当前光标指向哪个，因为可能一个元素都没有，所以为 option, 所有元素在entries中
    Dashboard{cursor: Option<usize>, entries: Vec<Entry>}, // 全局浏览窗口
    Help, // f1 help
    Details(UserInputEntry), // 某详情
    Creating{editing: Editing, u_input: UserInputEntry}, // 创建窗口
    DeleteTip(u32, String, Option<String>), // 删除时的弹窗, 显示名称和描述（可能有）
    Updating{editing: Editing, u_input: UserInputEntry, e_id: u32}, // 已有条目编辑窗口
    /// 要求键入主密码的窗口，载荷主密码输入string和准备进入的页面
    NeedMainPasswd(String, Box<Screen>, u8), // 要求键入主密码的窗口, u8 为重试次数
}

impl Screen {
    /// 表达该 屏幕 在进入前是否需要 主密码
    pub fn is_before_enter_need_main_pwd(&self) -> bool {
        match self { 
            // 需要主密码的屏幕在进入前不需要主密码，否则逻辑侧会无线递归
            // 因为被判定需要主密码的页面在进入前会渲染要求主密码的页面
            Screen::NeedMainPasswd(..) => false,
            // 主页和帮助页不需要
            Screen::Help | Screen::Dashboard {..} => false,
            _ => true,
        }
    }
    /// 新建编辑页面
    pub fn new_updating(u_input: UserInputEntry, e_id: u32) -> Self {
        Screen::Updating {editing: Editing::default(), u_input, e_id }
    }
    /// 新建新建页面
    pub fn new_creating() -> Self {
        Screen::Creating {editing: Editing::default(), u_input: UserInputEntry::default()}
    }
    
}

/// 表示正在编辑 UserInputEntry的 哪一个
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum Editing {
    #[default]
    Name,
    Description,
    Identity,
    Password,
}
