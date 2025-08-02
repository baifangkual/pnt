//! tui 运行时
//!
//! 处理 事件循环主要模块

use super::events::{Action, Event};
use crate::app::consts::APP_NAME_AND_VERSION;
use crate::app::context::SecurityContext;
use crate::app::entry::ValidEntry;
use crate::app::tui::TUIApp;
use crate::app::tui::components::EventHandler;
use crate::app::tui::components::Screen::{HomePageV1, InputMainPwd};
use crate::app::tui::intents::ScreenIntent;
use anyhow::{Context, Result, anyhow};
use arboard::Clipboard;
use crossterm::event::Event as CEvent;
use ratatui::crossterm;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::prelude::Alignment;
use std::collections::HashMap;

impl TUIApp {
    /// 返回上一个屏幕，
    /// 当上一个屏幕不存在时，发送 **退出** 事件
    pub fn back_screen(&mut self) {
        let pop_or = self.back_screen.pop();
        if let Some(p) = pop_or {
            self.screen = p;
            self.hot_msg.clear(); // 不同屏幕不同 hot_msg
        } else {
            self.send_action(Action::Quit)
        }
    }

    /// 处理需进入屏幕的需求,
    /// 该方法将当前屏幕（非InputMainPwd的）入栈并将当前screen切换为指定的，
    ///
    /// > 注：回退屏幕应使用back_screen而非该
    fn enter_screen_indent(&mut self, new_screen_intent: ScreenIntent) -> Result<()> {
        let new_screen = new_screen_intent.handle_intent(self)?;
        if new_screen.is_home_page() {
            // 若要进入的为 home_page，set 提示 version 和 help
            self.hot_msg.set_msg(
                &format!("| {} {} ", APP_NAME_AND_VERSION, "<F1> Help"),
                Some(5),
                Some(Alignment::Right),
                None,
            ); // tui 启动时显示一次的提示
        } else {
            self.hot_msg.clear();
        }
        if let InputMainPwd(_) = &self.screen {
            self.screen = new_screen; // NeedMainPasswd 屏幕直接切换，不入栈
        } else {
            let old_scr = std::mem::replace(&mut self.screen, new_screen);
            self.back_screen.push(old_scr);
        }

        Ok(())
    }

    #[inline]
    pub fn send_action(&self, action: Action) {
        self.event_queue.send(action);
    }
}

impl TUIApp {
    /// 事件处理入口
    pub fn invoke_handle_events(&mut self) -> Result<()> {
        let event = self.event_queue.next()?;
        match event {
            // 后端Crossterm事件
            Event::Crossterm(c_event) => match c_event {
                // 仅 按下, 这里或许过于严格了，或许放开仅 Press 情况
                CEvent::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    if let Some(action) = self.handle_key_press_event(key_event)? {
                        self.handle_action(action)
                    } else {
                        Ok(())
                    }
                }
                _ => Ok(()),
            },
            // tick 事件
            Event::Tick => self.tick(),
            // 封装的app 事件
            Event::App(action) => self.handle_action(action),
        }
    }

    /// action 处理
    fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::ScreenIntent(intent) => self.enter_screen_indent(intent)?,
            Action::EntryInsert(v_e) => self.insert_entry(&v_e),
            Action::EntryUpdate(v_e, e_id) => self.update_entry(&v_e, e_id),
            Action::EntryRemove(e_id) => self.remove_entry(e_id),
            Action::FlashTUIAppEncEntries => self.flash_tui_vec()?,
            Action::FlashHomePageDisplayEncEntries => self.flash_home_page_vec()?,
            Action::TurnOnFindMode => self.turn_on_find_mode()?,
            Action::TurnOffFindMode => self.turn_off_find_mode()?,
            Action::MainPwdVerifySuccess(sec_context) => self.hold_security_context(sec_context)?,
            Action::Quit => self.quit_tui_app(),
            Action::BackScreen => self.back_screen(),
            Action::Relock => self.relock(),
            Action::Actions(actions) => self.handle_actions(actions)?,
            Action::OptionYNTuiCallback(callback) => callback(self)?,
            Action::CopyToSysClipboard(info) => self.copy_to_sys_clip(info)?,
            Action::SetTuiHotMsg(msg, live_time, ali, color) => {
                self.hot_msg.set_msg(&msg, live_time, ali, color)
            }
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

    /// 将给定内容复制到系统剪贴板
    ///
    /// https://crates.io/crates/arboard
    pub fn copy_to_sys_clip(&self, info: String) -> Result<()> {
        Clipboard::new()?
            .set_text(info)
            .context("Failed to set clipboard contents")
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) -> Result<()> {
        self.idle_tick.idle_tick_increment();

        if self.idle_tick.need_relock() {
            self.relock();
        }
        if self.idle_tick.need_close() {
            self.quit_tui_app();
        }
        self.hot_msg.tick();
        Ok(())
    }

    /// 调用该方法，丢弃securityContext（重新锁定)，并回退屏幕到主页仪表盘
    ///
    /// 若并非unlock状态，则什么也不做
    fn relock(&mut self) {
        if self.context.is_verified() {
            self.context.security_context = None;
            // 屏幕回退
            while !self.screen.is_home_page() {
                self.back_screen();
            }
            if self.idle_tick.need_relock() {
                self.hot_msg.set_msg(
                    "[!] AUTO RELOCK (idle)",
                    Some(5),
                    Some(Alignment::Center),
                    None,
                );
            }
        }
    }

    pub fn quit_tui_app(&mut self) {
        self.running = false;
    }

    /// 向 db 删除一个 entry，并更新 store_entry_count - 1
    fn remove_entry(&mut self, e_id: u32) {
        self.context.storage.delete_entry(e_id);
        self.send_action(Action::FlashTUIAppEncEntries);
        self.send_action(Action::FlashHomePageDisplayEncEntries);
    }
    /// 向 db 添加一个 entry，并更新 store_entry_count + 1
    fn insert_entry(&mut self, e: &ValidEntry) {
        self.context.storage.insert_entry(e);
        self.send_action(Action::FlashTUIAppEncEntries);
        self.send_action(Action::FlashHomePageDisplayEncEntries);
    }

    fn update_entry(&mut self, e: &ValidEntry, e_id: u32) {
        self.context.storage.update_entry(e, e_id);
        self.send_action(Action::FlashTUIAppEncEntries);
        self.send_action(Action::FlashHomePageDisplayEncEntries);
    }

    /// 通过从db文件中重新查询以更新 tui-app hashmap中载荷的加密实体
    fn flash_tui_vec(&mut self) -> Result<()> {
        let enc_entries: HashMap<_, _> = self
            .context
            .storage
            .select_all_entry()
            .into_iter()
            .map(|e| (e.id, e))
            .collect();
        self.enc_entries = enc_entries;
        Ok(())
    }

    fn flash_home_page_vec(&mut self) -> Result<()> {
        if let HomePageV1(state) = &mut self.screen {
            state.reset_display_entries(self.enc_entries.values());
            Ok(())
        } else {
            Err(anyhow!("not home_page screen, no find mode"))
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
            self.send_action(Action::FlashHomePageDisplayEncEntries);
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
            self.enter_screen_indent(intent)?;
            Ok(())
        } else {
            Err(anyhow!("not NeedMainPasswd screen, no target screen"))
        }
    }
}
