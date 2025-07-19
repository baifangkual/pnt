//! tui 运行时
//!
//! 处理 事件循环主要模块

use super::event::key_ext::KeyEventExt;
use super::event::{Action, Event};
use crate::app::context::SecurityContext;
use crate::app::entry::ValidEntry;
use crate::app::tui::components::EventHandler;
use crate::app::tui::intents::ScreenIntent;
use crate::app::tui::screen::Screen::{HomePageV1, InputMainPwd};
use crate::app::tui::TUIApp;
use anyhow::{anyhow, Result};
use crossterm::event::Event as CEvent;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::prelude::Alignment;
use ratatui::{
    crossterm,
    crossterm::event::KeyEvent,
};

impl TUIApp {
    /// 返回上一个屏幕，
    /// 当上一个屏幕不存在时，发送 **退出** 事件
    pub fn back_screen(&mut self) {
        let pop_or = self.back_screen.pop();
        if let Some(p) = pop_or {
            self.screen = p;
            self.hot_msg.clear(); // 不同屏幕不同 hot_msg
        } else {
            self.send_app_event(Action::Quit)
        }
    }

    /// 处理需进入屏幕的需求
    fn handle_enter_screen_indent(&mut self, new_screen_intent: ScreenIntent) -> Result<()> {
        let new_screen = new_screen_intent.handle_intent(self)?;
        if let InputMainPwd(_) = &self.screen {
            self.screen = new_screen; // NeedMainPasswd 屏幕直接切换，不入栈
        } else {
            let old_scr = std::mem::replace(&mut self.screen, new_screen);
            self.back_screen.push(old_scr);
        }
        self.hot_msg.clear();
        Ok(())
    }

    #[inline]
    pub fn send_app_event(&self, event: Action) {
        self.events.send(event);
    }
}

impl TUIApp {
    /// 事件处理入口
    pub fn invoke_handle_events(&mut self) -> Result<()> {
        let event = self.events.next()?;
        match event {
            // tick 事件
            Event::Tick => self.tick(),
            // 后端Crossterm事件
            Event::Crossterm(event) => match event {
                // 仅 按下, 这里或许过于严格了，或许放开仅 Press 情况
                CEvent::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_press_event(key_event)?
                }
                _ => {}
            },
            // 封装的app 事件
            Event::App(action) => self.handle_action(action)?,
        }
        Ok(())
    }

    /// action 处理
    fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::ScreenIntent(intent) => self.handle_enter_screen_indent(intent)?,
            Action::EntryInsert(v_e) => self.insert_entry(&v_e),
            Action::EntryUpdate(v_e, e_id) => self.update_entry(&v_e, e_id),
            Action::EntryRemove(e_id) => self.remove_entry(e_id),
            Action::FlashVecItems(f) => self.flash_vec(f)?,
            Action::TurnOnFindMode => self.turn_on_find_mode()?,
            Action::TurnOffFindMode => self.turn_off_find_mode()?,
            Action::MainPwdVerifySuccess(sec_context) => self.hold_security_context(sec_context)?,
            Action::Quit => self.quit_tui_app(),
            Action::BackScreen => self.back_screen(),
            Action::RELOCK => self.re_lock(),
            Action::Actions(actions) => self.handle_actions(actions)?,
            Action::OptionYNTuiCallback(callback) => callback(self)?,
            Action::SetTuiHotMsg(msg, live_time, ali) => self.hot_msg.set_msg(&msg, live_time, ali),
        }
        Ok(())
    }

    /// 一组 action 处理
    fn handle_actions(&mut self, actions: Vec<Action>) -> Result<()> {
        for action in actions {
            self.handle_action(action)?;
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`TUIApp`].
    /// 按键事件处理，需注意，大写不一定表示按下shift，因为还有 caps Lock 键
    /// 进入该方法的 keyEvent.kind 一定为 按下 KeyEventKind::Press
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> Result<()> {
        // 每次操作将闲置tick计数清零
        self.idle_tick.reset_idle_tick_count();

        // 任何页面按 ctrl + c 都退出
        if key_event.is_ctrl_char('c') {
            self.send_app_event(Action::Quit);
            return Ok(());
        }
        // 按下 esc 的事件，将当前屏幕返回上一个屏幕，若当前为最后一个屏幕，则发送quit事件
        if key_event.is_esc() {
            self.handle_key_esc_event()?;
            return Ok(());
        }

        if let Some(action) = self.screen.handle_key_press_event(key_event)?{
            self.handle_action(action)?;
        }

        Ok(())
    }

    /// 按下 esc 的 处理器
    ///
    /// * home_page find 模式下 退出 find 模式
    /// * home_page find 输入框有值则清理值并重新查
    /// * 其他情况回退屏幕，无屏幕则发送退出事件
    pub fn handle_key_esc_event(&mut self) -> Result<()> {
        if let HomePageV1(state) = &mut self.screen {
            if state.find_mode() {
                self.send_app_event(Action::TurnOffFindMode);
            } else if !state.current_find_input().is_empty() {
                state.clear_find_input();
                self.send_app_event(Action::FlashVecItems(None))
            } else {
                self.back_screen();
            }
        } else {
            self.back_screen();
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {
        self.idle_tick.idle_tick_increment();

        if self.idle_tick.need_re_lock() {
            self.re_lock();
        }
        if self.idle_tick.need_close() {
            self.quit_tui_app();
        }

        self.hot_msg.tick()
    }

    /// 调用该方法，丢弃securityContext（重新锁定)，并回退屏幕到主页仪表盘
    ///
    /// 若并非unlock状态，则什么也不做
    fn re_lock(&mut self) {
        if self.context.is_verified() {
            self.context.security_context = None;
            // 屏幕回退
            while !self.screen.is_home_page() {
                self.back_screen();
            }
            if self.idle_tick.need_re_lock() {
                self.hot_msg
                    .set_msg("󰌾 AUTO RELOCK (idle)", Some(5), Some(Alignment::Center));
            }
        }
    }

    pub fn quit_tui_app(&mut self) {
        self.running = false;
    }

    /// 向 db 删除一个 entry，并更新 store_entry_count - 1
    fn remove_entry(&mut self, e_id: u32) {
        self.context.storage.delete_entry(e_id);
        self.send_app_event(Action::FlashVecItems(None));
    }
    /// 向 db 添加一个 entry，并更新 store_entry_count + 1
    fn insert_entry(&mut self, e: &ValidEntry) {
        self.context.storage.insert_entry(e);
        self.send_app_event(Action::FlashVecItems(None));
    }

    fn update_entry(&mut self, e: &ValidEntry, e_id: u32) {
        self.context.storage.update_entry(e, e_id);
        self.send_app_event(Action::FlashVecItems(None));
    }

    /// 当前页面为 home_page 时 刷新 home_page 的 vec 从库里重新拿
    /// 当不为 home_page时 Err
    /// 该方法会更新高亮行位置
    fn flash_vec(&mut self, find: Option<String>) -> Result<()> {
        if let HomePageV1(state) = &mut self.screen {
            let v_new = if let Some(f) = find {
                self.context.storage.select_entry_by_about_like(&f)
            } else {
                self.context.storage.select_all_entry()
            };
            state.reset_entries(v_new);
            Ok(())
        } else {
            Err(anyhow!("current screen is not home_page screen, cannot flash"))
        }
    }

    /// 开启 find mode
    fn turn_on_find_mode(&mut self) -> Result<()> {
        if let HomePageV1(state) = &mut self.screen {
            state.set_find_mode(true);
            Ok(())
        } else {
            Err(anyhow!("not home_page screen, no find mode"))
        }
    }

    /// 关闭 find mode
    fn turn_off_find_mode(&mut self) -> Result<()> {
        if let HomePageV1(state) = &mut self.screen {
            state.set_find_mode(false);
            // 获取 find_input 值，刷新vec
            if !state.current_find_input().is_empty() {
                let f = state.current_find_input().into();
                self.send_app_event(Action::FlashVecItems(Some(f)));
            } else {
                // 为空则全查
                // state.entries =  self.pnt.storage.select_all_entry();
                self.send_app_event(Action::FlashVecItems(None));
                // 刷新光标位置
            }
            Ok(())
        } else {
            Err(anyhow!("not home_page screen, no find mode"))
        }
    }

    /// 这是验证通过的事件处理终端方法
    /// 该方法内将使当前pnt上下文持有给定的SecurityContext,
    /// 并将当前屏幕切换为目标屏幕
    fn hold_security_context(&mut self, security_context: SecurityContext) -> Result<()> {
        if let InputMainPwd(state) = &mut self.screen {
            self.context.security_context = Some(security_context);
            let intent = state.take_target_screen()?;
            self.handle_enter_screen_indent(intent)?;
            Ok(())
        } else {
            Err(anyhow!("not NeedMainPasswd screen, no target screen"))
        }
    }

}
