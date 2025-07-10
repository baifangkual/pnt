mod event;
mod intents;
mod layout;
mod rt;
mod screen;
mod ui;
mod widgets;
mod colors;

use crate::app::context::PntContext;
use crate::app::tui::event::EventHandler;
use crate::app::tui::intents::EnterScreenIntent::ToDashBoardV1;
use crate::app::tui::rt::TUIApp;
use crate::app::tui::screen::Screen;
use crate::app::tui::screen::Screen::{DashboardV1, NeedMainPasswd};
use crate::app::tui::screen::states::{DashboardState, NeedMainPwdState};

/// tui 运行 模式
pub fn tui_run(pnt: PntContext) -> anyhow::Result<()> {
    let terminal = ratatui::init();
    let running = new_runtime(pnt);
    let result = running.run_main_loop(terminal);
    ratatui::restore();
    result
}

/// 新建 tui
fn new_runtime(pnt_context: PntContext) -> TUIApp {
    // tui 情况下 处理 要求立即密码的情况
    let screen = if pnt_context.cfg.need_main_passwd_on_run {
        NeedMainPasswd(NeedMainPwdState::new(ToDashBoardV1))
    } else {
        new_dashboard_screen(&pnt_context)
    };
    TUIApp {
        running: true,
        pnt: pnt_context,
        events: EventHandler::new(),
        screen,
        back_screen: Vec::with_capacity(10),
    }
}

/// tui 新建主页 主页面
fn new_dashboard_screen(context: &PntContext) -> Screen {
    let vec = context.storage.select_all_entry();
    DashboardV1(DashboardState::new(vec))
}
