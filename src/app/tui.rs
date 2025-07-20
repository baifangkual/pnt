mod colors;
mod components;
mod events;
mod intents;
mod layout;
mod rt;
mod ui;

use crate::app::cfg::InnerCfg;
use crate::app::consts::{APP_NAME, APP_NAME_AND_VERSION};
use crate::app::context::PntContext;
use crate::app::tui::colors::{
    CL_DDD_WHITE, CL_DD_WHITE,
};
use crate::app::tui::events::EventQueue;
use crate::app::tui::intents::ScreenIntent::ToHomePageV1;
use components::Screen;
use ratatui::prelude::{Alignment, Color};
use ratatui::DefaultTerminal;

/// tui 运行 模式
pub fn tui_run(pnt: PntContext) -> anyhow::Result<()> {
    let tui = new_runtime(pnt)?;
    let terminal = ratatui::init(); // 原始模式终端
    let result = tui.run_main_loop(terminal);
    ratatui::restore(); // 退出原始模式
    let tui = result?;
    // 因tick到期退出的，stdout告知
    if tui.idle_tick.need_close() {
        println!("{} auto closed with idle seconds", APP_NAME)
    }
    Ok(())
}

impl TUIApp {
    /// TUI程序主循环
    ///
    /// 该方法内 loop，返回Ok载荷self表示tui正常结束返回，载荷self可使后续行为访问内部状态等...
    ///
    /// 返回Err表示发生错误，该方法在返回前就关闭了连接了
    pub fn run_main_loop(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<TUIApp> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            match self.invoke_handle_events() {
                Ok(_) => (),
                Err(e) => {
                    self.quit_tui_app(); // 标记关闭状态, 下次main loop响应
                    self.context.storage.close(); // 有错误关闭数据库连接并退出当前方法
                    return Err(e);
                }
            }
        }
        Ok(self)
    }
}

/// 新建 tui
fn new_runtime(pnt_context: PntContext) -> anyhow::Result<TUIApp> {
    // tui 情况下 处理 要求立即密码的情况
    let (screen, hot_msg) = if pnt_context.is_need_mp_on_run() {
        let scr = Screen::new_input_main_pwd(ToHomePageV1, &pnt_context)?;
        let mut hm = HotMsg::new();
        hm.set_msg(
            &format!(
                "input main password to enter screen | {} {} ",
                APP_NAME_AND_VERSION, "<F1> Help"
            ),
            Some(255),
            Some(Alignment::Right),
            None,
        ); // tui 启动时显示一次的提示
        (scr, hm)
    } else {
        let scr = Screen::new_home_page1(&pnt_context);
        let mut hm = HotMsg::new();
        hm.set_msg(
            &format!("| {} {} ", APP_NAME_AND_VERSION, "<F1> Help"),
            Some(5),
            Some(Alignment::Right),
            None,
        ); // tui 启动时显示一次的提示
        (scr, hm)
    };

    let app = TUIApp {
        running: true,
        event_queue: EventQueue::new(),
        screen,
        back_screen: Vec::with_capacity(10),
        idle_tick: IdleTick::new(&pnt_context.cfg.inner_cfg),
        context: pnt_context,
        state_info: String::new(),
        hot_msg,
    };
    Ok(app)
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
    context: PntContext,
    /// Event handler.
    event_queue: EventQueue,
    /// 闲置tick计数，tick每秒一次
    idle_tick: IdleTick,
    /// 简单的 state info 信息，供页面渲染层显示，该字段面向渲染
    state_info: String,
    /// hot msg (tui界面底部bar显示临时信息，该字段面向渲染
    hot_msg: HotMsg,
}

struct HotMsg {
    temp_msg: Option<String>,
    temp_msg_alignment: Option<Alignment>,
    temp_msg_color: Option<Color>,
    /// temp_msg 存活时间 sec，响应tick，自减，为0则清除之
    temp_msg_live_countdown: u8,
    always_msg: String,
    always_msg_alignment: Option<Alignment>,
    always_msg_color: Option<Color>,
}
impl HotMsg {
    fn new() -> Self {
        Self {
            temp_msg: None,
            temp_msg_alignment: None,
            temp_msg_color: None,
            temp_msg_live_countdown: 0,
            always_msg: String::new(),
            always_msg_alignment: None,
            always_msg_color: None,
        }
    }

    /// 每次tick调用之，若temp存活时间到了，即将其清除
    ///
    /// 其 alignment及color也被清除（为不影响下一个temp_msg）
    fn tick(&mut self) {
        if self.temp_msg.is_some() {
            self.temp_msg_live_countdown = self.temp_msg_live_countdown.saturating_sub(1);
            if self.temp_msg_live_countdown == 0 {
                self.clear_temp_msg()
            }
        }
    }
    /// 设置消息，若给定 live_countdown 则为设置临时消息，
    /// 若无，则设置永久消息,
    /// 若align给定明确值，则将对应msg的alignment设定为对应值，否则msg的align为当前alignment
    fn set_msg(&mut self, msg: &str, live_countdown: Option<u8>, align: Option<Alignment>, color: Option<Color>) {
        if let Some(l) = live_countdown {
            self.set_temp_msg(msg, l, align, color);
        } else {
            self.set_always_msg(msg, align, color);
        }
    }
    /// 若当前hot_msg没有always_msg，设置居中的always_msg
    #[inline]
    fn set_always_if_none(&mut self, center_always_msg: &str) {
        if self.always_msg.is_empty() {
            self.set_always_msg(center_always_msg, None, None);
        }
    }

    fn clear_temp_msg(&mut self) {
        self.temp_msg = None;
        self.temp_msg_alignment = None;
        self.temp_msg_color = None;
    }

    /// 清理临时和永久消息
    fn clear(&mut self) {
        self.clear_temp_msg();
        self.clear_always_msg()
    }
    /// 设置临时消息，存活一定tick时间
    fn set_temp_msg(&mut self, temp_msg: &str, live_countdown: u8, align: Option<Alignment>, color: Option<Color>) {
        self.temp_msg = Some(temp_msg.to_string());
        self.temp_msg_live_countdown = live_countdown;
        self.temp_msg_alignment = align;
        self.temp_msg_color = color;
    }
    fn set_always_msg(&mut self, always_msg: &str, align: Option<Alignment>, color: Option<Color>) {
        self.always_msg = always_msg.to_string();
        self.always_msg_alignment = align;
        self.always_msg_color = color;
    }
    fn clear_always_msg(&mut self) {
        self.always_msg.clear();
        self.always_msg_alignment = None;
        self.always_msg_color = None;
    }
    /// 返回当前 msg，temp msg 优先于 always msg，
    /// 若当前无temp msg，则返回的为always_msg的
    fn msg(&self) -> &str {
        self.temp_msg.as_ref().unwrap_or(&self.always_msg)
    }
    /// 返回当前的调用 msg 返回的 msg 的 alignment
    fn alignment(&self) -> Alignment {
        if self.is_temp() {
            self.temp_msg_alignment.unwrap_or(Alignment::Center)
        } else {
            self.always_msg_alignment.unwrap_or(Alignment::Center)
        }
    }
    fn color(&self) -> Color {
        if self.is_temp() {
            self.temp_msg_color.unwrap_or(CL_DDD_WHITE)
        } else {
            self.always_msg_color.unwrap_or(CL_DD_WHITE)
        }
    }
    /// 返回当前调用 msg 方法时返回的 msg 类型，若temp_msg不为空，
    /// 则返回的 msg 为 temp_msg，即该方法返回 true，否则false
    fn is_temp(&self) -> bool {
        self.temp_msg.is_some()
    }
}

struct IdleTick {
    idle_tick_count: u32,
    auto_relock_idle_sec: u32,
    auto_close_idle_sec: u32,
}

impl IdleTick {
    fn new(inner_cfg: &InnerCfg) -> Self {
        // 0表示关闭，所以需要过滤掉0，设置为u32::MAX
        let auto_re_lk = inner_cfg
            .auto_relock_idle_sec
            .filter(|&sec| sec != 0)
            .unwrap_or(u32::MAX);
        // 0表示关闭，所以需要过滤掉0，设置为u32::MAX
        let auto_close = inner_cfg
            .auto_close_idle_sec
            .filter(|&sec| sec != 0)
            .unwrap_or(u32::MAX);
        Self {
            idle_tick_count: 0,
            auto_relock_idle_sec: auto_re_lk,
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
    fn need_relock(&self) -> bool {
        self.idle_tick_count > self.auto_relock_idle_sec
    }
    #[inline]
    fn need_close(&self) -> bool {
        self.idle_tick_count > self.auto_close_idle_sec
    }
}
