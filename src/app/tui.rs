mod colors;
mod event;
mod intents;
mod layout;
mod rt;
mod screen;
mod ui;
mod widgets;

use crate::app::cfg::InnerCfg;
use crate::app::context::PntContext;
use crate::app::tui::event::EventHandler;
use crate::app::tui::intents::EnterScreenIntent::ToHomePageV1;
use crate::app::tui::screen::Screen;
use crate::app::tui::screen::Screen::{HomePageV1, NeedMainPasswd};
use crate::app::tui::screen::states::{HomePageState, NeedMainPwdState};
use ratatui::DefaultTerminal;

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
    tick_adder: TickAdder,
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
        NeedMainPasswd(NeedMainPwdState::new(ToHomePageV1))
    } else {
        new_home_page_screen(&pnt_context)
    };
    TUIApp {
        running: true,
        events: EventHandler::new(),
        screen,
        back_screen: Vec::with_capacity(10),
        store_entry_count: pnt_context.storage.select_entry_count(),
        tick_adder: TickAdder::new(&pnt_context.cfg.inner_cfg),
        pnt: pnt_context,
    }
}

struct TickAdder {
    idle_tick_count: u32,
    auto_re_lock_idle_sec: u32,
    auto_close_idle_sec: u32,
}

impl TickAdder {
    fn new(inner_cfg: &InnerCfg) -> Self {
        // 0表示关闭，所以需要过滤掉0，设置为u32::MAX
        let auto_re_lk = inner_cfg
            .auto_re_lock_idle_sec
            .filter(|&sec| sec != 0)
            .unwrap_or(u32::MAX);
        // 0表示关闭，所以需要过滤掉0，设置为u32::MAX
        let auto_close = inner_cfg
            .auto_close_idle_sec
            .filter(|&sec| sec != 0)
            .unwrap_or(u32::MAX);
        Self {
            idle_tick_count: 0,
            auto_re_lock_idle_sec: auto_re_lk,
            auto_close_idle_sec: auto_close,
        }
    }

    #[inline]
    fn reset_idle_tick_count(&mut self) {
        self.idle_tick_count = 0;
    }

    #[inline]
    fn idle_tick_increment(&mut self) {
        // 使其最大不超过 u32max，最大值为u32max
        // auto... 关闭情况下值为u32max
        // 遂idle不会大于auto给定值，即关闭auto行为
        self.idle_tick_count = self.idle_tick_count.saturating_add(1)
    }
    #[inline]
    fn need_re_lock(&self) -> bool {
        self.idle_tick_count > self.auto_re_lock_idle_sec
    }
    #[inline]
    fn need_close(&self) -> bool {
        self.idle_tick_count > self.auto_close_idle_sec
    }
}

/// tui 新建主页 主页面
fn new_home_page_screen(context: &PntContext) -> Screen {
    let vec = context.storage.select_all_entry();
    HomePageV1(HomePageState::new(vec))
}
