mod colors;
mod event;
mod intents;
mod layout;
mod rt;
mod screen;
mod ui;
mod widgets;

use ratatui::DefaultTerminal;
use crate::app::context::PntContext;
use crate::app::tui::event::EventHandler;
use crate::app::tui::intents::EnterScreenIntent::ToDashBoardV1;
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

/// TUI Application.
pub struct TUIApp {
    /// Is the application running?
    running: bool,
    /// 当前屏幕
    screen: Screen,
    /// 上一个页面
    back_screen: Vec<Screen>,
    /// context
    pnt: PntContext,
    /// Event handler.
    events: EventHandler,
    /// current store entry count
    store_entry_count: u32,
    /// 闲置tick计数，tick每秒一次
    idle_tick_count: u32,
}

impl TUIApp {
    /// TUI程序主循环
    pub fn run_main_loop(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            match self.invoke_handle_events() {
                Ok(_) => (),
                Err(e) => {
                    self.quit_tui_app(); // 标记关闭状态, 下次main loop响应
                    self.pnt.storage.close(); // 有错误关闭数据库连接并退出当前方法
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

/// 新建 tui
fn new_runtime(pnt_context: PntContext) -> TUIApp {
    // tui 情况下 处理 要求立即密码的情况
    let screen = if pnt_context.is_need_mp_on_run() {
        NeedMainPasswd(NeedMainPwdState::new(ToDashBoardV1))
    } else {
        new_dashboard_screen(&pnt_context)
    };
    TUIApp {
        running: true,
        events: EventHandler::new(),
        screen,
        back_screen: Vec::with_capacity(10),
        store_entry_count: pnt_context.storage.select_entry_count(),
        pnt: pnt_context,
        idle_tick_count: 0
    }
}


/// tui 新建主页 主页面
fn new_dashboard_screen(context: &PntContext) -> Screen {
    let vec = context.storage.select_all_entry();
    DashboardV1(DashboardState::new(vec))
}


