use super::event::key_ext::KeyEventExt;
use super::event::{AppEvent, Event, EventHandler};
use crate::app::context::{PntContext, SecurityContext};
use crate::app::crypto::{Decrypter, NoEncrypter};
use crate::app::entry::{EncryptedEntry, InputEntry, ValidEntry};
use crate::app::tui::screen::Screen;
use crate::app::tui::screen::Screen::{
    Creating, Dashboard, DeleteTip, Details, Help, NeedMainPasswd, Updating,
};
use crate::app::tui::screen::options::OptionYN;
use crate::app::tui::screen::states::{DashboardState, Editing, NeedMainPwdState};
use anyhow::{Result, anyhow};
use crossterm::event::Event as CEvent;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::{
    DefaultTerminal, crossterm,
    crossterm::event::{KeyCode, KeyEvent},
};

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

    fn send_app_event(&self, event: AppEvent) {
        self.events.send(event);
    }
}

impl TUIRuntime {
    pub fn with_pnt(pnt_context: PntContext, (enc, dec): (NoEncrypter, NoEncrypter)) -> Self {
        let mut ve = pnt_context.storage.select_all_entry();
        ve.sort_by(EncryptedEntry::sort_by_update_time);
        Self {
            running: true,
            pnt: pnt_context,
            events: EventHandler::new(),
            screen: Dashboard(DashboardState::new(ve)), // Dashboard
            back_screen: Vec::with_capacity(10),
        }
    }

    /// TUI程序主循环
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
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
            AppEvent::FlashVecItems(f) => self.do_flash_vec(f)?,
            AppEvent::Quit => self.quit_tui_app(),
            AppEvent::TurnOnFindMode => self.turn_on_find_mode()?,
            AppEvent::TurnOffFindMode => self.turn_off_find_mode()?,
            AppEvent::MainPwdVerifySuccess(sec_context) => {
                self.hold_security_context_and_switch_to_target_screen(sec_context)?
            }
            AppEvent::MainPwdVerifyFailed => self.mp_retry_increment_or_err()?,
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
            self.handle_key_esc_event()?;
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
            Dashboard(state) => {
                // dashboard find
                if !state.find_mode {
                    if let KeyCode::Char('f') = key_event.code {
                        self.send_app_event(AppEvent::TurnOnFindMode);
                        return Ok(());
                    }
                    if key_event._is_q_ignore_case() {
                        self.back_screen();
                        return Ok(());
                    }
                    // 可进入 查看，编辑，删除tip，新建 页面
                    // 若当前光标无所指，则只能 创建
                    if let Some(c_ptr) = state.cursor_selected() {
                        let ptr_entry = &state.entries()[c_ptr];
                        // open
                        if key_event._is_o_ignore_case() || key_event._is_enter() {
                            // 解密
                            self.send_app_event(AppEvent::EnterScreen(Details(
                                // fixme 这里应修复进入需要密码页面前尝试解密的问题
                                //  因为当前 securityContext可能还不存在
                                ptr_entry.decrypt(self.pnt.try_encrypter()?)?,
                            )));
                            return Ok(());
                        }
                        // update
                        if key_event._is_u_ignore_case() {
                            // 解密
                            // fixme 这里应修复进入需要密码页面前尝试解密的问题
                            //  因为当前 securityContext可能还不存在
                            self.send_app_event(AppEvent::EnterScreen(Screen::new_updating(
                                ptr_entry.decrypt(self.pnt.try_encrypter()?)?,
                                ptr_entry.id,
                            )));
                            return Ok(());
                        }
                        // delete 但是dashboard 的光标？
                        // 任何删除都应确保删除页面上一级为dashboard
                        // 即非dashboard接收到删除事件时应确保关闭当前并打开删除
                        if key_event._is_d() {
                            self.send_app_event(AppEvent::EnterScreen(DeleteTip(
                                OptionYN::new_delete_tip(&ptr_entry),
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
                        self.send_app_event(AppEvent::EnterScreen(Screen::new_creating()));
                        return Ok(()); // fixed 拦截按键事件，下不处理，防止意外输入
                    }
                } else {
                    self.send_app_event(AppEvent::DoEditing(key_event.code));
                }
            }
            // 详情页
            Details(_) => {
                if key_event._is_q_ignore_case() {
                    self.back_screen();
                    return Ok(());
                }
                if key_event._is_d() {
                    // todo perf: details 页面应当有当前光标的id，不应该返回到dashboard再进行delete提示
                    self.back_screen(); // 回到 dashboard
                    self.enter_delete_cursor_pointing_entry_tips_screen()?;
                }
            }
            DeleteTip(option_yn) => {
                if key_event._is_q_ignore_case() {
                    self.back_screen();
                    return Ok(());
                }
                if let KeyCode::Char('y' | 'Y') | KeyCode::Enter = key_event.code {
                    self.send_app_event(AppEvent::EntryRemove(option_yn.content()?.id));
                    return Ok(());
                }
                if let KeyCode::Char('n' | 'N') = key_event.code {
                    self.back_screen();
                    return Ok(());
                }
            }
            Creating(state) | Updating(state) => {
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
                if key_event._is_ctrl_s() && state.current_input_validate() {
                    // 验证 todo 未通过验证应给予提示
                    // todo 应有 save tip 页面
                    match &self.screen {
                        Creating(state) => {
                            let (valid_e, _) = state.try_encrypt(self.pnt.try_encrypter()?)?;
                            self.send_app_event(AppEvent::EntryInsert(valid_e));
                        }
                        Updating(state) => {
                            let (valid_e, Some(e_id)) =
                                state.try_encrypt(self.pnt.try_encrypter()?)?
                            else {
                                return Err(anyhow!("updating entry must have an e_id"));
                            };
                            self.send_app_event(AppEvent::EntryUpdate(valid_e, e_id));
                        }
                        _ => {}
                    }
                    return Ok(()); // fixed 拦截按键事件，下不处理，防止意外输入
                }
                // 编辑窗口变化
                self.send_app_event(AppEvent::DoEditing(key_event.code));
            }
            // 需要主密码
            NeedMainPasswd(state) => {
                if key_event._is_enter() {
                    let verifier = self.pnt.build_mpv()?;
                    if verifier.verify(state.mp_input())? {
                        // 验证通过，发送 true 事件
                        let security_context = verifier.load_security_context(state.mp_input())?;
                        self.send_app_event(AppEvent::MainPwdVerifySuccess(security_context))
                    } else {
                        self.send_app_event(AppEvent::MainPwdVerifyFailed)
                    }
                    return Ok(()); // fixed 拦截按键事件，下不处理，防止意外输入
                }
                // 密码编辑窗口变化
                self.send_app_event(AppEvent::DoEditing(key_event.code));
            }
        }
        Ok(())
    }

    /// 按下 esc 的 处理器
    /// dashboard find 模式下 退出 find 模式
    pub fn handle_key_esc_event(&mut self) -> Result<()> {
        if let Dashboard(state) = &mut self.screen {
            if state.find_mode {
                // 解决 退出后 光标不见了，因为  state 没有存显示的实体们，
                // 但 退出的时候 find_input 里面有值，所以可使用 find_input
                // 里面的值重新刷一下 vec
                self.send_app_event(AppEvent::TurnOffFindMode);
            } else {
                self.back_screen();
            }
        } else {
            self.back_screen();
        }
        Ok(())
    }

    /// 处理进入某屏幕的情况，该方法内进行进入
    /// 若进入的屏幕需要 主密码
    /// 则会要求主密码页面
    /// 该方法是处理 AppEvent::EnterScreen 的端点，遂不应在进行信号发送
    fn handle_enter_screen_end_point(&mut self, new_screen: Screen) -> Result<()> {
        // 当前无主密码校验器且打开页面需要主密码，则证明未输入主密码或已过期，则需进入输入密码情况
        if self.pnt.security_context.is_none() && new_screen.is_before_enter_need_main_pwd() {
            self.enter_screen(NeedMainPasswd(NeedMainPwdState::new(new_screen)));
        } else {
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
        if let Dashboard(state) = &self.screen {
            if let Some(c_ptr) = state.cursor_selected() {
                let ptr_entry = &state.entries()[c_ptr];
                self.send_app_event(AppEvent::EnterScreen(DeleteTip(OptionYN::new_delete_tip(
                    &ptr_entry,
                ))));
            } else {
                return Err(anyhow!("current cursor is pointing none"));
            }
        } else {
            return Err(anyhow!("current screen is not dashboard screen"));
        }
        Ok(())
    }

    /// 处理光标向上事件
    fn cursor_up(&mut self) {
        if let Dashboard(state) = &mut self.screen {
            if let Some(p) = state.cursor_selected() {
                if p == 0 {
                    state.update_cursor(Some(state.entry_count() - 1))
                } else {
                    state.cursor.select_previous();
                }
            }
        } else if let Creating(state) | Updating(state) = &mut self.screen {
            state.cursor_up();
        }
    }

    /// 处理光标向下事件
    fn cursor_down(&mut self) {
        if let Dashboard(state) = &mut self.screen {
            if let Some(p) = state.cursor_selected() {
                if p >= state.entry_count() - 1 {
                    state.update_cursor(Some(0))
                } else {
                    state.cursor.select_next();
                }
            }
        } else if let Creating(state) | Updating(state) = &mut self.screen {
            state.cursor_down();
        }
    }

    pub fn quit_tui_app(&mut self) {
        self.running = false;
    }

    fn do_editing(&mut self, key_code: KeyCode) -> Result<()> {
        if let Creating(state) | Updating(state) = &mut self.screen {
            // 不为 desc 的 响应 enter 到下一行
            if Editing::Description != *state.current_editing_type() {
                if let KeyCode::Enter = key_code {
                    self.send_app_event(AppEvent::CursorDown);
                    return Ok(());
                }
            }
            // do editing...
            let input = state.current_editing_string_mut();
            match key_code {
                KeyCode::Backspace => {
                    input.pop();
                    ()
                }
                // KeyCode::Left => {} // todo 左移光标
                // KeyCode::Right => {} // todo 右移光标
                // KeyCode::BackTab => {} // todo up
                KeyCode::Char(value) => input.push(value),
                KeyCode::Enter => input.push('\n'),
                _ => {}
            }
            Ok(())
        } else if let NeedMainPasswd(state) = &mut self.screen {
            match key_code {
                KeyCode::Backspace => {
                    state.mp_input.pop();
                    ()
                }
                // KeyCode::Left => {} // todo 左移光标
                // KeyCode::Right => {} // todo 右移光标
                KeyCode::Char(value) => state.mp_input.push(value),
                _ => {}
            }
            Ok(())
        } else if let Dashboard(state) = &mut self.screen {
            match key_code {
                KeyCode::Backspace => {
                    state.find_input.pop();
                    ()
                }
                // KeyCode::Left => {} // todo 左移光标
                // KeyCode::Right => {} // todo 右移光标
                KeyCode::Char(value) => state.find_input.push(value),
                _ => {}
            }
            Ok(())
        } else {
            Err(anyhow!("current screen is no do_editing event"))
        }
    }

    fn do_insert(&mut self, e: &ValidEntry) {
        self.pnt.storage.insert_entry(&e);
        self.back_screen();
        self.send_app_event(AppEvent::FlashVecItems(None));
    }

    fn do_update(&mut self, e: &ValidEntry, e_id: u32) {
        self.pnt.storage.update_entry(&e, e_id);
        self.back_screen();
        self.send_app_event(AppEvent::FlashVecItems(None));
    }

    /// 当前页面为 dashboard 时 刷新 dashboard 的 vec 从库里重新拿
    /// 当不为 dashboard时 Err
    /// 该方法会更新高亮行位置
    fn do_flash_vec(&mut self, find: Option<String>) -> Result<()> {
        if let Dashboard(state) = &mut self.screen {
            state.entries = if let Some(f) = find {
                self.pnt.storage.select_entry_by_name_like(&f)
            } else {
                self.pnt.storage.select_all_entry()
            };
            // 按最后更新时间倒叙
            state.entries.sort_by(EncryptedEntry::sort_by_update_time);
            if !state.entries.is_empty() {
                if let None = state.cursor_selected() {
                    state.update_cursor(Some(0))
                }
            } else {
                state.update_cursor(None);
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
        self.send_app_event(AppEvent::FlashVecItems(None));
    }

    /// 开启 find mode
    fn turn_on_find_mode(&mut self) -> Result<()> {
        if let Dashboard(state) = &mut self.screen {
            state.find_mode = true;
            Ok(())
        } else {
            Err(anyhow!("not Dashboard screen, no find mode"))
        }
    }

    /// 关闭 find mode
    fn turn_off_find_mode(&mut self) -> Result<()> {
        if let Dashboard(state) = &mut self.screen {
            state.find_mode = false;
            // 获取 find_input 值，刷新vec
            if !state.find_input.is_empty() {
                let f = state.find_input.clone();
                self.send_app_event(AppEvent::FlashVecItems(Some(f)));
            } else {
                // 为空则全查
                // state.entries =  self.pnt.storage.select_all_entry();
                self.send_app_event(AppEvent::FlashVecItems(None));
                // 刷新光标位置
            }
            Ok(())
        } else {
            Err(anyhow!("not Dashboard screen, no find mode"))
        }
    }

    /// 这是验证通过的事件处理终端方法
    /// 该方法内将使当前pnt上下文持有给定的SecurityContext,
    /// 并将当前屏幕切换为目标屏幕
    fn hold_security_context_and_switch_to_target_screen(
        &mut self,
        security_context: SecurityContext,
    ) -> Result<()> {
        if let NeedMainPasswd(state) = &mut self.screen {
            self.pnt.security_context = Some(security_context);
            self.screen = state.take_target_screen()?;
            // 不能使用 EnterScreen事件进入屏幕，因为该事件的处理者会将老屏幕压入栈
            // 但当前为 NeedMainPasswd屏幕，所以压栈进去无意义，
            // 遂该方法内应直接进行屏幕替换即可
            // self.send_app_event(AppEvent::EnterScreen(state.take_target_screen()?));
            Ok(())
        } else {
            Err(anyhow!("not NeedMainPasswd screen, no target screen"))
        }
    }

    /// 验证失败的事件，自增或err
    fn mp_retry_increment_or_err(&mut self) -> Result<()> {
        if let NeedMainPasswd(state) = &mut self.screen {
            state.increment_retry_count()
        } else {
            Err(anyhow!("current is not NeedMainPasswd screen"))
        }
    }
}
