mod colors;
mod event;
mod intents;
mod layout;
mod rt;
mod screen;
mod ui;
mod widgets;

use crate::app::cfg::InnerCfg;
use crate::app::consts::{APP_NAME, APP_NAME_AND_VERSION};
use crate::app::context::PntContext;
use crate::app::tui::event::EventHandler;
use crate::app::tui::intents::EnterScreenIntent::ToHomePageV1;
use crate::app::tui::screen::Screen;
use crate::app::tui::screen::Screen::{HomePageV1, NeedMainPasswd};
use crate::app::tui::screen::states::{HomePageState, NeedMainPwdState};
use ratatui::DefaultTerminal;
use ratatui::prelude::Alignment;

/// tui 运行 模式
pub fn tui_run(pnt: PntContext) -> anyhow::Result<()> {
    let terminal = ratatui::init();
    let running = new_runtime(pnt);
    let result = running.run_main_loop(terminal);
    ratatui::restore();
    let app_after_run = result?;
    // 因tick到期退出的，stdout告知
    if app_after_run.idle_tick.need_close() {
        println!("{} app auto closed with idle seconds", APP_NAME)
    }
    Ok(())
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
    idle_tick: IdleTick,
    /// hot msg (tui界面底部bar显示临时信息
    hot_msg: HotMsg,
}

struct HotMsg {
    temp_msg: Option<String>,
    temp_msg_alignment: Alignment,
    /// temp_msg 存活时间 sec，响应tick，自减，为0则清除之
    temp_msg_live_countdown: u8,
    always_msg: String,
    always_msg_alignment: Alignment,
}
impl HotMsg {
    fn new() -> Self {
        Self {
            temp_msg: None,
            temp_msg_alignment: Alignment::Center,
            temp_msg_live_countdown: 0,
            always_msg: String::new(),
            always_msg_alignment: Alignment::Center,
        }
    }

    /// 每次tick调用之，若temp存活时间到了，即将其清除
    fn tick(&mut self) {
        if self.temp_msg.is_some() {
            self.temp_msg_live_countdown = self.temp_msg_live_countdown.saturating_sub(1);
            if self.temp_msg_live_countdown == 0 {
                self.temp_msg = None;
            }
        }
    }
    /// 设置消息，若给定 live_countdown 则为设置临时消息，
    /// 若无，则设置永久消息,
    /// 若align给定明确值，则将对应msg的alignment设定为对应值，否则msg的align为当前alignment
    fn set_msg(&mut self, msg: &str, live_countdown: Option<u8>, align: Option<Alignment>) {
        if let Some(l) = live_countdown {
            self.set_temp_msg(msg, l, align.unwrap_or(self.temp_msg_alignment));
        } else {
            self.set_always_msg(msg, align.unwrap_or(self.always_msg_alignment));
        }
    }
    /// 若当前hot_msg没有always_msg，设置居中的always_msg
    #[inline]
    fn set_if_not_always(&mut self, center_always_msg: &str) {
        if self.always_msg.is_empty() {
            self.set_always_msg(center_always_msg, Alignment::Center);
        }
    }

    /// 清理临时和永久消息
    fn clear(&mut self) {
        self.temp_msg = None;
        self.clear_always_msg()
    }
    /// 设置临时消息，存活一定tick时间
    fn set_temp_msg(&mut self, temp_msg: &str, live_countdown: u8, align: Alignment) {
        self.temp_msg = Some(temp_msg.to_string());
        self.temp_msg_live_countdown = live_countdown;
        self.temp_msg_alignment = align;
    }
    fn set_always_msg(&mut self, always_msg: &str, align: Alignment) {
        self.always_msg = always_msg.to_string();
        self.always_msg_alignment = align;
    }
    fn clear_always_msg(&mut self) {
        self.always_msg.clear();
    }
    /// 返回当前 msg，temp msg 优先于 always msg，
    /// 若当前无temp msg，则返回的为always_msg的
    fn msg(&self) -> &str {
        self.temp_msg.as_ref().unwrap_or(&self.always_msg)
    }
    /// 返回当前的调用 msg 返回的 msg 的 alignment
    fn msg_alignment(&self) -> Alignment {
        if self.is_temp_msg() {
            self.temp_msg_alignment
        } else {
            self.always_msg_alignment
        }
    }
    /// 返回当前调用 msg 方法时返回的 msg 类型，若temp_msg不为空，
    /// 则返回的 msg 为 temp_msg，即该方法返回 true，否则false
    fn is_temp_msg(&self) -> bool {
        self.temp_msg.is_some()
    }
}

impl TUIApp {
    /// TUI程序主循环
    pub fn run_main_loop(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<TUIApp> {
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
        Ok(self)
    }
}

/// 新建 tui
fn new_runtime(pnt_context: PntContext) -> TUIApp {
    // tui 情况下 处理 要求立即密码的情况
    let (screen, hot_msg) = if pnt_context.is_need_mp_on_run() {
        let scr = NeedMainPasswd(NeedMainPwdState::new(ToHomePageV1));
        let mut hm = HotMsg::new();
        hm.set_msg(
            &format!(
                "input main password to enter screen | {} {} ",
                APP_NAME_AND_VERSION, "<F1> Help"
            ),
            Some(255),
            Some(Alignment::Right),
        );
        (scr, hm)
    } else {
        let scr = new_home_page_screen(&pnt_context);
        let mut hm = HotMsg::new();
        hm.set_msg(
            &format!("| {} {} ", APP_NAME_AND_VERSION, "<F1> Help"),
            Some(5),
            Some(Alignment::Right),
        );
        (scr, hm)
    };

    TUIApp {
        running: true,
        events: EventHandler::new(),
        screen,
        back_screen: Vec::with_capacity(10),
        store_entry_count: pnt_context.storage.select_entry_count(),
        idle_tick: IdleTick::new(&pnt_context.cfg.inner_cfg),
        pnt: pnt_context,
        hot_msg,
    }
}

struct IdleTick {
    idle_tick_count: u32,
    auto_re_lock_idle_sec: u32,
    auto_close_idle_sec: u32,
}

impl IdleTick {
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
