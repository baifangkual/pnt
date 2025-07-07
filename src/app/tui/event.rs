pub mod key_ext;

use super::screen::Screen;
use crate::app::context::SecurityContext;
use crate::app::entry::ValidEntry;
use anyhow::{Context, Result};
use ratatui::crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use crate::app::tui::intents::EnterScreenIntent;

/// The frequency at which tick events are emitted.
/// 每秒一次
const TICK_FPS: f64 = 1.0;

/// Representation of all possible events.
pub enum Event {
    /// 由子线程发送的固定频率的事件，[`TICK_FPS`]
    Tick,
    /// Crossterm events.
    ///
    /// These events are emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Application events.
    ///
    /// Use this event to emit custom events that are specific to your application.
    App(AppEvent),
}

/// Application events.
///
/// You can extend this enum with your own custom events.
pub enum AppEvent {
    EnterScreenIntent(EnterScreenIntent), // 仅描述意图要进入的页面
    TurnOnFindMode,
    TurnOffFindMode,
    CursorUp,
    CursorDown,
    DoEditing(KeyCode),
    EntryInsert(ValidEntry), // 插入必要全局刷新 vec，因为插入到库前还不知道id
    EntryUpdate(ValidEntry, u32), // u32 为 id
    EntryRemove(u32), // u32 为 id
    FlashVecItems(Option<String>), // 在dashboard 刷新 载荷 entries 的 vec，若该携带Some，则使用其中str做查询
    MainPwdVerifyFailed,
    MainPwdVerifySuccess(SecurityContext),
    Quit,
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
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
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&self, app_event: AppEvent) {
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

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    fn run(self) -> Result<()> {
        let tick_interval = Duration::from_secs_f64(1.0 / TICK_FPS);
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
            if event::poll(timeout).context("failed to poll for crossterm events")? {
                // 该子线程消费 终端键盘事件并向 tui 更新线程发送键盘事件
                let event = event::read().context("failed to read crossterm event")?;
                self.send(Event::Crossterm(event));
            }
        }
    }

    /// Sends an event to the receiver.
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        // 忽略发送错误，程序关闭时线程drop接收者，这会返回Err，正常情况
        // 不使用 let _ 会有烦人提示
        let _ = self.sender.send(event);
    }
}