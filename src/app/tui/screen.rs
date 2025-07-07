pub(crate) mod states;
pub(crate) mod options;

use crate::app::entry::{EncryptedEntry, InputEntry};



/// 当前屏幕
#[derive(Debug)] // todo 后续应移除重的 Clone
pub enum Screen {
    /// 当前光标指向哪个，因为可能一个元素都没有，所以为 option, 所有元素在entries中
    Dashboard(DashboardState), // 全局浏览窗口
    Help,                    // f1 help
    Details(InputEntry), // 某详情
    Edit(EditingState), // 创建窗口
    // Updating (EditingState), // 已有条目编辑窗口
    DeleteTip(OptionYN<EncryptedEntry>), // 删除时的弹窗, 显示名称和描述（可能有）
    // SaveTip(OptionYN<EditingState>), // 保存前提示窗口
    // 修改主密码窗口
    /// 要求键入主密码的窗口，载荷主密码输入string和准备进入的页面
    NeedMainPasswd(NeedMainPwdState), // 要求键入主密码的窗口, u8 为重试次数
}

impl Screen {
    /// 表达该 屏幕 在进入前是否需要 主密码
    pub fn is_before_enter_need_main_pwd(&self) -> bool {
        match self {
            // 需要主密码的屏幕在进入前不需要主密码，否则逻辑侧会无线递归
            // 因为被判定需要主密码的页面在进入前会渲染要求主密码的页面
            Screen::NeedMainPasswd(..) => false,
            // 主页和帮助页不需要
            Screen::Help | Screen::Dashboard { .. } => false,
            _ => true,
        }
    }
    /// 新建编辑页面
    pub fn new_updating(u_input: InputEntry, e_id: u32) -> Self {
        Screen::Edit(EditingState::new_updating(u_input, e_id))
    }
    /// 新建新建页面
    pub fn new_creating() -> Self {
        Screen::Edit(Default::default())
    }
}

use states::DashboardState;
use crate::app::tui::screen::options::OptionYN;
use crate::app::tui::screen::states::{EditingState, NeedMainPwdState};
