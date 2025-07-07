mod layout;
mod ui;
mod event;
mod runtime;
mod screen;
mod widgets;
mod intents;

use crate::app::context::PntContext;
use crate::app::entry::EncryptedEntry;
use crate::app::tui::event::EventHandler;
use crate::app::tui::intents::EnterScreenIntent::ToDashBoard;
use crate::app::tui::runtime::TUIRuntime;
use crate::app::tui::screen::states::{DashboardState, NeedMainPwdState};
use crate::app::tui::screen::Screen;
use crate::app::tui::screen::Screen::{Dashboard, NeedMainPasswd};

/// tui 运行 模式
pub fn tui_run(pnt: PntContext) -> anyhow::Result<()> {
    let terminal = ratatui::init();
    let running = new_runtime(pnt);
    let result = running.run_main_loop(terminal);
    ratatui::restore();
    result
}


/// 新建 tui
fn new_runtime(pnt_context: PntContext) -> TUIRuntime {
    // tui 情况下 处理 要求立即密码的情况
    let screen = if pnt_context.cfg.need_main_passwd_on_run {
        NeedMainPasswd(NeedMainPwdState::new(ToDashBoard))
    } else {
        new_dashboard_screen(&pnt_context)
    };
    TUIRuntime {
        running: true,
        pnt: pnt_context,
        events: EventHandler::new(),
        screen,
        back_screen: Vec::with_capacity(10),
    }
}

/// tui 新建主页 主页面
fn new_dashboard_screen(context: &PntContext) -> Screen {
    let mut vec = context.storage.select_all_entry();
    vec.sort_unstable_by(EncryptedEntry::sort_by_update_time);
    Dashboard(DashboardState::new(vec))
}

