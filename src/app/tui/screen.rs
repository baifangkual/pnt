pub(crate) mod states;
pub(crate) mod options;

use crate::app::entry::{EncryptedEntry, InputEntry};



/// 当前屏幕
pub enum Screen {
    /// 当前光标指向哪个，因为可能一个元素都没有，所以为 option, 所有元素在entries中
    Dashboard(DashboardState), // 全局浏览窗口
    Help,                    // f1 help
    Details(InputEntry, u32), // 某详情, u32 为 id
    Edit(EditingState), // 创建窗口
    // Updating (EditingState), // 已有条目编辑窗口
    YNTip(YNState<EncryptedEntry>), // y/n 弹窗
    // SaveTip(OptionYN<EditingState>), // 保存前提示窗口
    // 修改主密码窗口
    /// 要求键入主密码的窗口，载荷主密码输入string和准备进入的页面
    NeedMainPasswd(NeedMainPwdState), // 要求键入主密码的窗口, u8 为重试次数
}

impl Screen {
    
    /// 表达该屏幕是否为最上级的dashboard
    ///
    /// > 该方法给多个可能实现的 dashboard 做准备
    pub fn is_dashboard(&self) -> bool {
        match self {
            Screen::Dashboard(..) => true,
            _ => false,
        }
    }

    /// 新建编辑页面
    pub fn new_edit_updating(u_input: InputEntry, e_id: u32) -> Self {
        Screen::Edit(EditingState::new_updating(u_input, e_id))
    }
    /// 新建新建页面
    pub fn new_edit_creating() -> Self {
        Screen::Edit(Default::default())
    }
}

use states::DashboardState;
use crate::app::tui::screen::options::YNState;
use crate::app::tui::screen::states::{EditingState, NeedMainPwdState};
