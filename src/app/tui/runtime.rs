use super::event::key_ext::KeyEventExt;
use super::event::{AppEvent, Event, EventHandler};
use crate::app::consts::{MAIN_PASS_KEY, MAIN_PASS_MAX_RE_TRY};
use crate::app::context::PntContext;
use crate::app::encrypt::{Encrypter, MainPwdVerifier, NoEncrypter};
use crate::app::entry::{Entry, UserInputEntry, ValidInsertEntry};
use crate::app::storage::sqlite::SqliteConn;
use crate::app::tui::screen::Screen::{
    Creating, Dashboard, DeleteTip, Details, Help, NeedMainPasswd, Updating,
};
use crate::app::tui::screen::{Editing, Screen};
use anyhow::{Result, anyhow, Error};
use crossterm::event::Event as CEvent;
use log::__private_api::enabled;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::{
    DefaultTerminal, crossterm,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use crate::app::tui::error::TError;

/// TUI Application.
pub struct TUIRuntime {
    /// Is the application running?
    pub running: bool,
    /// 当前屏幕
    pub screen: Screen,
    /// 上一个页面
    pub back_screen: Vec<Screen>,
    /// context
    pub pnt: PntContext,
    /// Event handler.
    pub events: EventHandler,
    pub encrypter: NoEncrypter,
    pub decrypter: NoEncrypter,
}

impl TUIRuntime {
    /// 返回上一个屏幕，
    /// 当上一个屏幕不存在时，发送 **退出** 事件
    fn back_screen(&mut self) {
        let pop_or = self.back_screen.pop();
        if let Some(p) = pop_or {
            self.screen = p;
        } else {
            self.send_app_event(AppEvent::Quit)
        }
    }
    /// 将当前屏幕替换为给定的屏幕，并将旧屏幕入栈，
    fn enter_screen(&mut self, new_screen: Screen) {
        // 因为所有权的问题，无法将 screen先入栈再将新屏幕赋值给其，
        // 遂使用std::mem::replace
        let old_scr = std::mem::replace(&mut self.screen, new_screen);
        self.back_screen.push(old_scr);
    }

    fn send_app_event(&mut self, event: AppEvent) {
        self.events.send(event);
    }
}

impl TUIRuntime {
    pub fn with_pnt(pnt_context: PntContext, (enc, dec): (NoEncrypter, NoEncrypter)) -> Self {
        let mut ve = pnt_context.storage.select_all_entry();
        ve.sort_by(Entry::sort_by_update_time);
        let c_ptr = if ve.is_empty() { None } else { Some(0) };
        Self {
            running: true,
            pnt: pnt_context,
            events: EventHandler::new(),
            screen: Dashboard {
                cursor: c_ptr,
                entries: ve,
            }, // Dashboard
            back_screen: Vec::with_capacity(10),
            encrypter: enc,
            decrypter: dec,
        }
    }

    /// TUI程序主循环
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.handle_events() {
                Ok(_) => {}
                Err(e) => {
                    self.quit_tui_app();
                    self.pnt.storage.close(); // 有错误关闭数据库连接并退出当前方法
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// 事件处理入口
    pub fn handle_events(&mut self) -> Result<()> {
        let event = self.events.next()?;
        match event {
            // tick 事件
            Event::Tick => self.tick(),
            // 后端Crossterm事件
            Event::Crossterm(event) => match event {
                // 仅 按下
                CEvent::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_press_event(key_event)?
                }
                _ => {}
            },
            // 封装的app 事件
            Event::App(app_event) => self.handle_app_event(app_event)?,
        }
        Ok(())
    }

    /// APP Event 处理
    fn handle_app_event(&mut self, app_event: AppEvent) -> Result<()> {
        match app_event {
            AppEvent::EnterScreen(target_sc) => self.handle_enter_screen_end_point(target_sc)?,
            AppEvent::CursorUp => self.cursor_up(),
            AppEvent::CursorDown => self.cursor_down(),
            AppEvent::DoEditing(code) => self.do_editing(code)?,
            AppEvent::EntryInsert(v_e) => self.do_insert(&v_e),
            AppEvent::EntryUpdate(v_e, e_id) => self.do_update(&v_e, e_id),
            AppEvent::EntryRemove(e_id) => self.do_remove(e_id),
            AppEvent::FlashVec => self.do_flash_vec()?,
            AppEvent::Quit => self.quit_tui_app(),
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`TUIRuntime`].
    /// 按键事件处理，需注意，大写不一定表示按下shift，因为还有 caps Lock 键
    /// 进入该方法的 keyEvent.kind 一定为 按下 KeyEventKind::Press
    pub fn handle_key_press_event(&mut self, key_event: KeyEvent) -> Result<()> {
        // 任何页面按 ctrl + c 都退出
        if key_event._is_ctrl_c() {
            self.send_app_event(AppEvent::Quit);
            return Ok(());
        }
        // 按下 esc 的事件，将当前屏幕返回上一个屏幕，若当前为最后一个屏幕，则发送quit事件
        if key_event._is_esc() {
            self.back_screen();
            return Ok(());
        }
        // f1 按下 进入 帮助页面
        if key_event._is_f1() {
            self.send_app_event(AppEvent::EnterScreen(Help));
            return Ok(());
        }

        // 不同屏幕不同按键响应，包装为不同的app事件
        // 走到此，则 ctrl + c ，quit， f1 已被处理，
        // 遂下无
        match &self.screen {
            // help 页面
            Help => {
                if key_event._is_q_ignore_case() {
                    self.back_screen();
                    return Ok(());
                }
            }
            // 仪表盘
            Dashboard { cursor, entries } => {
                if key_event._is_q_ignore_case() {
                    self.back_screen();
                    return Ok(());
                }
                // 可进入 查看，编辑，删除tip，新建 页面
                // 若当前光标无所指，则只能 创建
                if let Some(c_ptr) = cursor {
                    let ptr_entry = &entries[*c_ptr];
                    // open
                    if key_event._is_o_ignore_case() || key_event._is_enter() {
                        let ve =
                            UserInputEntry::decrypt_from_entry(&self.decrypter, ptr_entry.clone());
                        self.send_app_event(AppEvent::EnterScreen(Details(ve)));
                        return Ok(());
                    }
                    // update
                    if key_event._is_u_ignore_case() {
                        let ve =
                            UserInputEntry::decrypt_from_entry(&self.decrypter, ptr_entry.clone());
                        self.send_app_event(AppEvent::EnterScreen(Screen::new_updating(
                            ve,
                            ptr_entry.id,
                        )));
                        return Ok(());
                    }
                    // delete 但是dashboard 的光标？
                    // 任何删除都应确保删除页面上一级为dashboard
                    // 即非dashboard接收到删除事件时应确保关闭当前并打开删除
                    if key_event._is_d() {
                        self.send_app_event(AppEvent::EnterScreen(DeleteTip(
                            ptr_entry.id,
                            ptr_entry.name.clone(),
                            ptr_entry.description.clone(),
                        )));
                        return Ok(());
                    }
                    // 上移
                    if key_event._is_k() || key_event._is_up() {
                        self.send_app_event(AppEvent::CursorUp);
                        return Ok(());
                    }
                    // 下移
                    if key_event._is_down() || key_event._is_j() {
                        self.send_app_event(AppEvent::CursorDown);
                        return Ok(());
                    }
                }
                // 任意光标位置都可以新建
                if key_event._is_i_ignore_case() {
                    self.send_app_event(AppEvent::EnterScreen(Screen::new_creating()))
                }
            }
            // 详情页
            Details(_) => {
                if key_event._is_q_ignore_case() {
                    self.back_screen();
                    return Ok(());
                }
                if key_event._is_d() {
                    self.back_screen(); // 回到 dashboard
                    self.enter_delete_cursor_pointing_entry_tips_screen()?;
                }
            }
            Creating {
                editing: _,
                u_input,
            } => {
                // 上移
                if key_event._is_up() {
                    self.send_app_event(AppEvent::CursorUp);
                    return Ok(());
                }
                // 下移
                if key_event._is_down() || key_event._is_tab() {
                    self.send_app_event(AppEvent::CursorDown);
                    return Ok(());
                }
                // 保存
                if key_event._is_ctrl_s() {
                    // 验证 todo 未通过验证应给予提示
                    if u_input.validate() {
                        // clone ... todo 或可优化非 clone
                        let entry = u_input.clone().encrypt(&self.encrypter);
                        // todo 应有 save tip 页面
                        self.send_app_event(AppEvent::EntryInsert(entry));
                    }
                }
                // 编辑窗口变化
                self.send_app_event(AppEvent::DoEditing(key_event.code));
            }
            DeleteTip(e_id, ..) => {
                if key_event._is_q_ignore_case() {
                    self.back_screen();
                    return Ok(());
                }
                if let KeyCode::Char('y' | 'Y') | KeyCode::Enter = key_event.code {
                    self.send_app_event(AppEvent::EntryRemove(*e_id));
                    return Ok(());
                }
                if let KeyCode::Char('n' | 'N') = key_event.code {
                    self.back_screen();
                    return Ok(());
                }
            }
            Updating {
                editing: _,
                u_input,
                e_id,
            } => {
                // 上移
                if key_event._is_up() {
                    self.send_app_event(AppEvent::CursorUp);
                    return Ok(());
                }
                // 下移
                if key_event._is_down() || key_event._is_tab() {
                    self.send_app_event(AppEvent::CursorDown);
                    return Ok(());
                }
                // 保存
                if key_event._is_ctrl_s() {
                    // 验证 todo 未通过验证应给予提示
                    if u_input.validate() {
                        // clone ... todo 或可优化非 clone
                        let entry = u_input.clone().encrypt(&self.encrypter);
                        // todo 应有 save tip 页面
                        self.send_app_event(AppEvent::EntryUpdate(entry, *e_id));
                    }
                }
                // 编辑窗口变化
                self.send_app_event(AppEvent::DoEditing(key_event.code));
            }
            // 需要主密码
            NeedMainPasswd(mp, next_enter, re_try) => {
                // 重试失败到一定次数 则 直接 向方法栈上层传递 Err，
                // 主事件处理循环处会关闭数据库连接并退出
                if *re_try >= MAIN_PASS_MAX_RE_TRY {
                    return Err(Error::from(TError::ReTryMaxExceed(*re_try)));
                }

                if key_event._is_enter() {
                    let mp_hash_b64d = self.pnt.storage.select_cfg_v_by_key(MAIN_PASS_KEY).unwrap();
                    let verifier =
                        MainPwdVerifier::from_salt_and_passwd(&self.pnt.cfg.salt, mp_hash_b64d);
                    // todo 换为 非 clone
                    if verifier.verify(mp.clone()) {
                        // 验证通过，发送 true 事件
                        self.screen = *next_enter.clone();
                        return Ok(());
                    } else {
                        self.send_app_event(AppEvent::EnterScreen(NeedMainPasswd(
                            String::new(),
                            next_enter.clone(),
                            re_try + 1,
                        )))
                    }
                }
                // 密码编辑窗口变化
                self.send_app_event(AppEvent::DoEditing(key_event.code));
            }
        }

        // match key_event.code {
        //     // esc or q - to event quit
        //     KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
        //     // ctrl + c - to event quit
        //     KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
        //         self.events.send(AppEvent::Quit)
        //     }
        //     _ => {}
        // }
        Ok(())
    }

    /// 处理进入某屏幕的情况，该方法内进行进入
    /// 若进入的屏幕需要 主密码
    /// 则会要求主密码页面
    /// 该方法是处理 AppEvent::EnterScreen 的端点，遂不应在进行信号发送
    fn handle_enter_screen_end_point(&mut self, new_screen: Screen) -> Result<()> {
        // 当前无主密码校验器且打开页面需要主密码，则证明未输入主密码或已过期，则需进入输入密码情况
        if self.pnt.mpv.is_none() && new_screen.is_before_enter_need_main_pwd() {
            self.enter_screen(NeedMainPasswd(String::new(), Box::new(new_screen), 0));
        } else {
            // 主密码重试会进入该块，直接进入，并且 re_try + 1
            self.enter_screen(new_screen);
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {
        // 可用判定当前包含被解密的字段的窗口打开的时间，
        // 超过一定阈值则发送关闭子窗口的事件
    }

    /// 要求进入光标指向的当前entry的删除提示页面
    /// 若光标当前指向不为Some或当前screen不是dashboard，返回Err
    fn enter_delete_cursor_pointing_entry_tips_screen(&mut self) -> Result<()> {
        if let Dashboard { cursor, entries } = &self.screen {
            if let Some(c_ptr) = cursor {
                let ptr_entry = &entries[*c_ptr];
                self.send_app_event(AppEvent::EnterScreen(DeleteTip(
                    ptr_entry.id,
                    ptr_entry.name.clone(),
                    ptr_entry.description.clone(),
                )));
            } else {
                return Err(anyhow!("current cursor is pointing none"));
            }
        } else {
            return Err(anyhow!("current screen is not dashboard screen"));
        }
        Ok(())
    }

    fn cursor_up(&mut self) {
        if let Dashboard { cursor, entries } = &mut self.screen {
            if let Some(c_ptr) = cursor {
                if *c_ptr == 0 {
                    *c_ptr = entries.len() - 1;
                } else {
                    *c_ptr -= 1;
                }
            }
        } else if let Creating { editing, .. } | Updating { editing, .. } = &mut self.screen {
            match editing {
                Editing::Name => *editing = Editing::Password,
                Editing::Description => *editing = Editing::Name,
                Editing::Identity => *editing = Editing::Description,
                Editing::Password => *editing = Editing::Identity,
            }
        }
    }
    fn cursor_down(&mut self) {
        if let Dashboard { cursor, entries } = &mut self.screen {
            if let Some(c_ptr) = cursor {
                if *c_ptr == entries.len() - 1 {
                    *c_ptr = 0;
                } else {
                    *c_ptr += 1;
                }
            }
        } else if let Creating { editing, .. } | Updating { editing, .. } = &mut self.screen {
            match editing {
                Editing::Name => *editing = Editing::Description,
                Editing::Description => *editing = Editing::Identity,
                Editing::Identity => *editing = Editing::Password,
                Editing::Password => *editing = Editing::Name,
            }
        }
    }

    pub fn quit_tui_app(&mut self) {
        self.running = false;
    }

    fn do_editing(&mut self, key_code: KeyCode) -> Result<()> {
        if let Creating { editing, u_input }
        | Updating {
            editing, u_input, ..
        } = &mut self.screen
        {
            // 不为 desc 的 响应 enter 到下一行
            if Editing::Description != *editing {
                if let KeyCode::Enter = key_code {
                    self.send_app_event(AppEvent::CursorDown);
                    return Ok(());
                }
            }
            // do editing...
            let input = match &editing {
                Editing::Name => &mut u_input.name,
                Editing::Description => &mut u_input.description,
                Editing::Identity => &mut u_input.identity,
                Editing::Password => &mut u_input.password,
            };
            match key_code {
                KeyCode::Backspace => {
                    input.pop();
                    ()
                }
                // KeyCode::Left => {} // todo 左移光标
                // KeyCode::Right => {} // todo 右移光标
                // KeyCode::BackTab => {} // todo up
                KeyCode::Char(value) => input.push(value),
                _ => {}
            }
            Ok(())
        } else if let NeedMainPasswd(mp, ..) = &mut self.screen {
            match key_code {
                KeyCode::Backspace => {
                    mp.pop();
                    ()
                }
                // KeyCode::Left => {} // todo 左移光标
                // KeyCode::Right => {} // todo 右移光标
                KeyCode::Char(value) => mp.push(value),
                _ => {}
            }
            Ok(())
        } else {
            Err(anyhow!(
                "current screen is not Creating or Updating or NeedMainPasswd screen"
            ))
        }
    }

    fn do_insert(&mut self, e: &ValidInsertEntry) {
        self.pnt.storage.insert_entry(&e);
        self.back_screen();
        self.send_app_event(AppEvent::FlashVec);
    }

    fn do_update(&mut self, e: &ValidInsertEntry, e_id: u32) {
        self.pnt.storage.update_entry(&e, e_id);
        self.back_screen();
        self.send_app_event(AppEvent::FlashVec);
    }

    /// 当前页面为 dashboard 时 刷新 dashboard 的 vec 从库里重新拿
    /// 当不为 dashboard时 Err
    fn do_flash_vec(&mut self) -> Result<()> {
        if let Dashboard { cursor, entries } = &mut self.screen {
            entries.clear();
            entries.append(&mut self.pnt.storage.select_all_entry());
            entries.sort_by(Entry::sort_by_update_time);
            if !entries.is_empty() {
                *cursor = Some(0)
            } else {
                *cursor = None;
            }
            Ok(())
        } else {
            Err(anyhow!(
                "current screen is not dashboard screen, cannot flash"
            ))
        }
    }

    fn do_remove(&mut self, e_id: u32) {
        self.pnt.storage.delete_entry(e_id);
        self.back_screen();
        self.send_app_event(AppEvent::FlashVec);
    }
}
