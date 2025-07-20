use crate::app::context::SecurityContext;
use crate::app::entry::ValidEntry;
use crate::app::tui::intents::ScreenIntent;
use anyhow::Result;
use ratatui::crossterm::event::{self, Event as CEvent};
use ratatui::prelude::{Alignment, Color};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// Representation of all possible events.
pub enum Event {
    /// 由子线程发送的固定频率的事件，[`TICK_FPS`]
    Tick,
    /// Crossterm events.
    ///
    /// These events are emitted by the terminal.
    Crossterm(CEvent),
    /// Application events.
    ///
    /// Use this event to emit custom events that are specific to your application.
    App(Action),
}

/// app 的 行为
pub enum Action {
    /// 多个顺序的行为
    Actions(Vec<Action>),
    /// 仅描述意图要进入的页面
    ScreenIntent(ScreenIntent),
    /// 回退到上一个屏幕
    BackScreen,
    /// 重新锁定
    Relock,
    /// optionYn tui回调
    OptionYNTuiCallback(crate::app::tui::components::yn::FnCallYN),
    /// 设定TUI hot msg, 该结构内包含信息，持续时间，位置
    SetTuiHotMsg(String, Option<u8>, Option<Alignment>, Option<Color>),
    TurnOnFindMode,
    TurnOffFindMode,
    /// 新的加密实体插入，插入必要全局刷新 vec，因为插入到库前还不知道id
    EntryInsert(ValidEntry),
    /// 更新加密实体，u32为id
    EntryUpdate(ValidEntry, u32),
    /// 删除加密实体，u32为id
    EntryRemove(u32),
    /// 在home_page 刷新 载荷 entries 的 vec，若该携带Some，则使用其中str做查询
    FlashVecItems(Option<String>),
    /// 主密码校验成功时会载荷 securityContext
    MainPwdVerifySuccess(SecurityContext),
    /// 复制内容到系统剪贴板
    CopyToSysClipboard(String),
    /// tui程序退出
    Quit,
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventQueue {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl EventQueue {
    /// Constructs a new instance of [`EventQueue`] and spawns a new thread to handle events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let actor = EventThread::new(sender.clone());
        thread::spawn(|| actor.run());
        Self { sender, receiver }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    ///
    /// 读取一个事件，该方法阻塞直到事件可用
    #[inline]
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&self, app_event: Action) {
        // 忽略发送错误，程序关闭时线程drop接收者，这会返回Err，正常情况
        // 不使用 let _ 会有烦人提示
        let _ = self.sender.send(Event::App(app_event));
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventThread {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
}

impl EventThread {
    /// Constructs a new instance of [`EventThread`].
    fn new(sender: mpsc::Sender<Event>) -> Self {
        Self { sender }
    }
    /// The frequency at which tick events are emitted.
    /// 每秒一次
    const TICK_FPS: f64 = 1.0;
    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    fn run(self) -> Result<()> {
        let tick_interval = Duration::from_secs_f64(1.0 / Self::TICK_FPS);
        let mut last_tick = Instant::now();
        loop {
            // emit tick events at a fixed rate
            let timeout = tick_interval.saturating_sub(last_tick.elapsed());
            // 固频发送 TICK
            if timeout == Duration::ZERO {
                last_tick = Instant::now();
                self.send(Event::Tick);
            }
            // poll for crossterm events, ensuring that we don't block the tick interval
            if event::poll(timeout)? {
                // 该子线程消费 终端键盘事件并向 tui 更新线程发送键盘事件
                let event = event::read()?;
                self.send(Event::Crossterm(event));
            }
        }
    }

    /// Sends an event to the receiver.
    #[inline]
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        // 忽略发送错误，程序关闭时线程drop接收者，这会返回Err，正常情况
        // 不使用 let _ 会有烦人提示
        let _ = self.sender.send(event);
    }
}
