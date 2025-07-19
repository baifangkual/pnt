pub(crate) mod states;
pub(crate) mod yn;

use crate::app::context::PntContext;
use crate::app::entry::InputEntry;
use crate::app::tui::screen::states::{EditingState, NeedMainPwdState};
use crate::app::tui::screen::yn::YNState;
use ratatui::widgets::ListState;
use states::HomePageState;

/// 当前屏幕
pub enum Screen {
    /// 当前光标指向哪个，因为可能一个元素都没有，所以为 option, 所有元素在entries中
    HomePageV1(HomePageState),
    /// f1 help, list state 为行光标状态
    Help(ListState),
    /// 某详情, u32 为 id
    Details(InputEntry, u32),
    /// 编辑窗口
    Edit(EditingState),
    /// y/n 弹窗
    YNOption(YNState),
    /// 要求键入主密码的窗口，载荷主密码输入string和准备进入的页面
    NeedMainPasswd(NeedMainPwdState),
}

impl Screen {
    /// 表达该屏幕是否为最上级的home_page
    ///
    /// > 该方法给多个可能实现的 home_page 做准备
    pub fn is_home_page(&self) -> bool {
        matches!(self, Screen::HomePageV1(..))
    }

    pub fn is_help(&self) -> bool {
        matches!(self, Screen::Help(..))
    }

    /// 新建编辑页面
    pub fn new_edit_updating(u_input: InputEntry, e_id: u32) -> Self {
        Screen::Edit(EditingState::new_updating(u_input, e_id))
    }
    /// 新建新建页面
    pub fn new_edit_creating() -> Self {
        Screen::Edit(EditingState::new_creating())
    }

    pub fn new_help() -> Self {
        Screen::Help(ListState::default())
    }

    /// tui 新建主页 主页面
    pub fn new_home_page1(context: &PntContext) -> Self {
        let vec = context.storage.select_all_entry();
        Screen::HomePageV1(HomePageState::new(vec))
    }
}
