use super::event::key_ext::KeyEventExt;
use super::event::{AppEvent, Event, EventHandler};
use crate::app::context::{PntContext, SecurityContext};
use crate::app::entry::ValidEntry;
use crate::app::tui::intents::EnterScreenIntent;
use crate::app::tui::intents::EnterScreenIntent::{
    ToDeleteYNOption, ToDetail, ToEditing, ToHelp, ToSaveYNOption,
};
use crate::app::tui::screen::Screen;
use crate::app::tui::screen::Screen::{DashboardV1, Details, Edit, Help, NeedMainPasswd, YNOption};
use crate::app::tui::screen::states::Editing;
use anyhow::{Result, anyhow};
use crossterm::event::Event as CEvent;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::{
    DefaultTerminal, crossterm,
    crossterm::event::{KeyCode, KeyEvent},
};

/// TUI Application.
pub struct TUIApp {
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

impl TUIApp {
    /// 返回上一个屏幕，
    /// 当上一个屏幕不存在时，发送 **退出** 事件
    pub fn back_screen(&mut self) {
        let pop_or = self.back_screen.pop();
        if let Some(p) = pop_or {
            self.screen = p;
        } else {
            self.send_app_event(AppEvent::Quit)
        }
    }

    /// 切换屏幕，push_old_screen 为 true 表示将换下来的屏幕压入back栈中
    fn change_current_screen(&mut self, new_screen: Screen, push_old_screen: bool) -> Result<()> {
        if push_old_screen {
            let old_scr = std::mem::replace(&mut self.screen, new_screen);
            Ok(self.back_screen.push(old_scr))
        } else {
            Ok(self.screen = new_screen)
        }
    }

    /// 处理需进入屏幕的需求
    fn handle_enter_screen_indent(&mut self, new_screen_intent: EnterScreenIntent) -> Result<()> {
        let new_screen = new_screen_intent.handle_intent(&self)?;
        if let NeedMainPasswd(_) = &self.screen {
            Ok(self.change_current_screen(new_screen, false)?)
        } else {
            Ok(self.change_current_screen(new_screen, true)?)
        }
    }

    pub fn send_app_event(&self, event: AppEvent) {
        self.events.send(event);
    }
}

impl TUIApp {
    /// TUI程序主循环
    pub fn run_main_loop(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            match self.invoke_handle_events() {
                Ok(_) => (),
                Err(e) => {
                    self.quit_tui_app(); // 标记关闭状态
                    self.pnt.storage.close(); // 有错误关闭数据库连接并退出当前方法
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// 事件处理入口
    fn invoke_handle_events(&mut self) -> Result<()> {
        let event = self.events.next()?;
        match event {
            // tick 事件
            Event::Tick => self.invoke_handle_tick(),
            // 后端Crossterm事件
            Event::Crossterm(event) => match event {
                // 仅 按下
                CEvent::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.invoke_current_screen_handle_key_press_event(key_event)?
                }
                _ => {}
            },
            // 封装的app 事件
            Event::App(app_event) => self.invoke_handle_app_event(app_event)?,
        }
        Ok(())
    }

    /// APP Event 处理
    fn invoke_handle_app_event(&mut self, app_event: AppEvent) -> Result<()> {
        match app_event {
            AppEvent::CursorUp => self.cursor_up(),
            AppEvent::CursorDown => self.cursor_down(),
            AppEvent::EnterScreenIntent(intent) => self.handle_enter_screen_indent(intent)?,
            AppEvent::EntryInsert(v_e) => self.do_insert(&v_e),
            AppEvent::EntryUpdate(v_e, e_id) => self.do_update(&v_e, e_id),
            AppEvent::EntryRemove(e_id) => self.do_remove(e_id),
            AppEvent::FlashVecItems(f) => self.do_flash_vec(f)?,
            AppEvent::TurnOnFindMode => self.turn_on_find_mode()?,
            AppEvent::TurnOffFindMode => self.turn_off_find_mode()?,
            AppEvent::MainPwdVerifySuccess(sec_context) => {
                self.hold_security_context_and_switch_to_target_screen(sec_context)?
            }
            AppEvent::MainPwdVerifyFailed => self.mp_retry_increment_or_err()?,
            AppEvent::Quit => self.quit_tui_app(),
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`TUIApp`].
    /// 按键事件处理，需注意，大写不一定表示按下shift，因为还有 caps Lock 键
    /// 进入该方法的 keyEvent.kind 一定为 按下 KeyEventKind::Press
    fn invoke_current_screen_handle_key_press_event(&mut self, key_event: KeyEvent) -> Result<()> {
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
            self.send_app_event(AppEvent::EnterScreenIntent(ToHelp));
            return Ok(());
        }

        // 不同屏幕不同按键响应，包装为不同的app事件
        // 走到此，则 ctrl + c ，quit， f1 已被处理，
        // 遂下无
        match &mut self.screen {
            // help 页面
            Help => {
                if key_event._is_q_ignore_case() {
                    self.back_screen();
                    return Ok(());
                }
            }
            // 仪表盘
            DashboardV1(state) => {
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
                        let curr_ptr_e_id = state.entries()[c_ptr].id;
                        // open
                        if key_event._is_o_ignore_case() || key_event._is_enter() {
                            self.send_app_event(AppEvent::EnterScreenIntent(ToDetail(
                                curr_ptr_e_id,
                            )));
                            return Ok(());
                        }
                        // update
                        if key_event._is_u_ignore_case() {
                            self.send_app_event(AppEvent::EnterScreenIntent(ToEditing(Some(
                                curr_ptr_e_id,
                            ))));
                            return Ok(());
                        }
                        // delete 但是dashboard 的光标？
                        // 任何删除都应确保删除页面上一级为dashboard
                        // 即非dashboard接收到删除事件时应确保关闭当前并打开删除
                        if key_event._is_d() {
                            self.send_app_event(AppEvent::EnterScreenIntent(ToDeleteYNOption(
                                curr_ptr_e_id,
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
                        self.send_app_event(AppEvent::EnterScreenIntent(ToEditing(None)));
                        return Ok(()); // fixed 拦截按键事件，下不处理，防止意外输入
                    }
                } else {
                    self.do_editing_key_event(key_event)?;
                }
            }
            // 详情页
            Details(_, e_id) => {
                if key_event._is_q_ignore_case() {
                    self.back_screen();
                    return Ok(());
                }
                if key_event._is_d() {
                    let de_id = *e_id;
                    self.send_app_event(AppEvent::EnterScreenIntent(ToDeleteYNOption(de_id)));
                    return Ok(());
                }
            }
            // 弹窗页面
            YNOption(option_yn) => {
                if key_event._is_q_ignore_case() {
                    self.back_screen();
                    return Ok(());
                }
                if let KeyCode::Char('y' | 'Y') | KeyCode::Enter = key_event.code {
                    return if let Some(y_call) = option_yn.take_y_call() {
                        y_call(self)
                    } else {
                        Err(anyhow!("not found y-call"))
                    };
                }
                if let KeyCode::Char('n' | 'N') = key_event.code {
                    return if let Some(n_call) = option_yn.take_n_call() {
                        n_call(self)
                    } else {
                        Err(anyhow!("not found n-call"))
                    };
                }
            }
            Edit(state) => {
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
                    let e_id = state.current_e_id();
                    // 该处已修改：该处不加密，只有 save tip 页面 按下 y 才触发 加密并保存
                    let input_entry = state.current_input_entry().clone();
                    self.send_app_event(AppEvent::EnterScreenIntent(ToSaveYNOption(
                        input_entry,
                        e_id,
                    )));
                    return Ok(()); // fixed 拦截按键事件，下不处理，防止意外输入
                }
                // 编辑窗口变化
                self.do_editing_key_event(key_event)?;
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
                self.do_editing_key_event(key_event)?;
            }
        }
        Ok(())
    }

    /// 按下 esc 的 处理器
    ///
    /// * dashboard find 模式下 退出 find 模式
    /// * dashboard find 输入框有值则清理值并重新查
    /// * 其他情况回退屏幕，无屏幕则发送退出事件
    pub fn handle_key_esc_event(&mut self) -> Result<()> {
        if let DashboardV1(state) = &mut self.screen {
            if state.find_mode {
                self.send_app_event(AppEvent::TurnOffFindMode);
            } else if !state.current_find_input().is_empty() {
                state.clear_find_input();
                self.send_app_event(AppEvent::FlashVecItems(None))
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
    pub fn invoke_handle_tick(&self) {
        // 可用判定当前包含被解密的字段的窗口打开的时间，
        // 超过一定阈值则发送关闭子窗口的事件
    }

    /// 处理光标向上事件
    fn cursor_up(&mut self) {
        if let DashboardV1(state) = &mut self.screen {
            state.cursor_up();
        } else if let Edit(state) = &mut self.screen {
            state.cursor_up();
        }
    }

    /// 处理光标向下事件
    fn cursor_down(&mut self) {
        if let DashboardV1(state) = &mut self.screen {
            state.cursor_down();
        } else if let Edit(state) = &mut self.screen {
            state.cursor_down();
        }
    }

    pub fn quit_tui_app(&mut self) {
        self.running = false;
    }

    fn do_editing_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        if let Edit(state) = &mut self.screen {
            // 不为 desc 的 响应 enter 到下一行
            if Editing::Notes != *state.current_editing_type() {
                if let KeyCode::Enter = key_event.code {
                    self.send_app_event(AppEvent::CursorDown);
                    return Ok(());
                }
            }
            // do editing...
            let input = state.current_editing_string_mut();
            match key_event.code {
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
            match key_event.code {
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
        } else if let DashboardV1(state) = &mut self.screen {
            // let c_event = CEvent::Key(key_event); // 临时构建 由 key向上整个 CEvent以匹配handle_event方法签名
            match key_event.code {
                KeyCode::Enter => self.send_app_event(AppEvent::TurnOffFindMode),
                _ => {
                    // 返回bool表示是否修改了，暂时用不到
                   let _ = state.find_textarea().input(key_event);
                },
            }
            Ok(())
        } else {
            Err(anyhow!("current screen is no do_editing event"))
        }
    }

    fn do_insert(&mut self, e: &ValidEntry) {
        self.pnt.storage.insert_entry(&e);
        self.send_app_event(AppEvent::FlashVecItems(None));
    }

    fn do_update(&mut self, e: &ValidEntry, e_id: u32) {
        self.pnt.storage.update_entry(&e, e_id);
        self.send_app_event(AppEvent::FlashVecItems(None));
    }

    /// 当前页面为 dashboard 时 刷新 dashboard 的 vec 从库里重新拿
    /// 当不为 dashboard时 Err
    /// 该方法会更新高亮行位置
    fn do_flash_vec(&mut self, find: Option<String>) -> Result<()> {
        if let DashboardV1(state) = &mut self.screen {
            let v_new = if let Some(f) = find {
                self.pnt.storage.select_entry_by_about_like(&f)
            } else {
                self.pnt.storage.select_all_entry()
            };
            state.reset_entries(v_new);
            Ok(())
        } else {
            Err(anyhow!(
                "current screen is not dashboard screen, cannot flash"
            ))
        }
    }

    fn do_remove(&mut self, e_id: u32) {
        self.pnt.storage.delete_entry(e_id);
        self.send_app_event(AppEvent::FlashVecItems(None));
    }

    /// 开启 find mode
    fn turn_on_find_mode(&mut self) -> Result<()> {
        if let DashboardV1(state) = &mut self.screen {
            state.find_mode = true;
            Ok(())
        } else {
            Err(anyhow!("not Dashboard screen, no find mode"))
        }
    }

    /// 关闭 find mode
    fn turn_off_find_mode(&mut self) -> Result<()> {
        if let DashboardV1(state) = &mut self.screen {
            state.find_mode = false;
            // 获取 find_input 值，刷新vec
            if !state.current_find_input().is_empty() {
                let f = state.current_find_input().into();
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
            let intent = state.take_target_screen()?;
            self.handle_enter_screen_indent(intent)?;
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
