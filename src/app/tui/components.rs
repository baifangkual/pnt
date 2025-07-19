//! 组件，构成tui元素，能够响应事件

use crate::app::tui::event::key_ext::KeyEventExt;
use crate::app::tui::event::Action;
use crate::app::tui::intents::ScreenIntent::{ToDeleteYNOption, ToDetail, ToEditing, ToHelp, ToSaveYNOption};
use crate::app::tui::screen::states::Editing;
use crate::app::tui::screen::Screen;
use crate::app::tui::screen::Screen::{Details, Edit, Help, HomePageV1, InputMainPwd, YNOption};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Alignment;

pub trait EventHandler {
    /// 响应 key 按下事件，要求self的可变引用，即对事件的响应可能改变自身状态，
    /// 方法返回 Option<Action>，即响应可能产出对应行为要求
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Action>>;
}

#[inline]
fn ok_action(action: Action) -> anyhow::Result<Option<Action>> {
    Ok(Some(action))
}
#[inline]
fn ok_none() -> anyhow::Result<Option<Action>> {
    Ok(None)
}

impl EventHandler for Screen {
    
    
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Action>> {
        match self {
            // help 页面
            Help(list_cursor) => {
                if key_event.is_char('q') {
                    return ok_action(Action::BackScreen);
                }
                // 上移
                if key_event.is_char('k') || key_event.is_up() {
                    list_cursor.select_previous();
                    return ok_none();
                }
                // 下移
                if key_event.is_down() || key_event.is_char('j') {
                    list_cursor.select_next();
                    return ok_none();
                }
                // 顶部 底部
                if let KeyCode::Char('g') | KeyCode::Home = key_event.code {
                    list_cursor.select_first();
                    return ok_none();
                }
                // 顶部 底部
                if let KeyCode::Char('G') | KeyCode::End = key_event.code {
                    list_cursor.select_last();
                    return ok_none();
                }
                return ok_none();
            }
            // 仪表盘
            HomePageV1(state) => {
                // f1 按下 进入 帮助页面
                if key_event.is_f1() {
                    return ok_action(Action::ScreenIntent(ToHelp));
                }

                // home_page find
                if !state.find_mode() {
                    if key_event.is_char('f') {
                        return ok_action(Action::TurnOnFindMode);
                    }
                    // 响应 按下 l 丢弃 securityContext以重新锁定
                    if key_event.is_char('l') {
                        return ok_action(Action::RELOCK);
                    }
                    if key_event.is_char('q') {
                        return ok_action(Action::BackScreen);
                    }
                    // 可进入 查看，编辑，删除tip，新建 页面
                    // 若当前光标无所指，则只能 创建
                    if let Some(c_ptr) = state.cursor_selected() {
                        let curr_ptr_e_id = state.entries()[c_ptr].id;
                        // open
                        if key_event.is_char('o') || key_event.is_enter() {
                            return ok_action(Action::ScreenIntent(ToDetail(curr_ptr_e_id)));
                        }
                        // edit
                        if key_event.is_char('e') {
                            return ok_action(Action::ScreenIntent(ToEditing(Some(curr_ptr_e_id))));
                        }
                        // delete 但是home_page 的光标？
                        // 任何删除都应确保删除页面上一级为home_page
                        // 即非home_page接收到删除事件时应确保关闭当前并打开删除
                        if key_event.is_char('d') {
                            return ok_action(Action::ScreenIntent(ToDeleteYNOption(curr_ptr_e_id)));
                        }
                        // 上移
                        if key_event.is_char('k') || key_event.is_up() {
                            state.cursor_up();
                            return ok_none();
                        }
                        // 下移
                        if key_event.is_down() || key_event.is_char('j') {
                            state.cursor_down();
                            return ok_none();
                        }
                        // 顶部 底部
                        if let KeyCode::Char('g') | KeyCode::Home = key_event.code {
                            state.cursor_mut_ref().select_first();
                            return ok_none();
                        }
                        // 顶部 底部
                        if let KeyCode::Char('G') | KeyCode::End = key_event.code {
                            state.cursor_mut_ref().select_last();
                            return ok_none();
                        }
                    }
                    // 任意光标位置都可以新建
                    if key_event.is_char('a') {
                        return ok_action(Action::ScreenIntent(ToEditing(None)));
                    }
                    ok_none()
                } else {
                    match key_event.code {
                        KeyCode::Enter => ok_action(Action::TurnOffFindMode),
                        _ => {
                            // 返回bool表示是否修改了，暂时用不到
                            let _ = state.find_textarea().input(key_event);
                            ok_none()
                        }
                    }
                }
            }
            // 详情页
            Details(_, e_id) => {
                // f1 按下 进入 帮助页面
                if key_event.is_f1() {
                    return ok_action(Action::ScreenIntent(ToHelp));
                }

                if key_event.is_char('q') {
                    return ok_action(Action::BackScreen);
                }
                if key_event.is_char('d') {
                    let de_id = *e_id;
                    return ok_action(Action::ScreenIntent(ToDeleteYNOption(de_id)));
                }
                if key_event.is_char('l') {
                  return ok_action(Action::RELOCK);
                }
                ok_none()
            }
            // 弹窗页面
            YNOption(option_yn) => {
                if key_event.is_char('q') {
                    return ok_action(Action::BackScreen);
                }
                if let KeyCode::Char('y' | 'Y') | KeyCode::Enter = key_event.code {
                    return if let Some(y_call) = option_yn.take_y_call() {
                        ok_action(Action::OptionYNTuiCallback(y_call))
                    } else {
                        unreachable!("选项必须设定y_callback")
                    };
                }
                if let KeyCode::Char('n' | 'N') = key_event.code {
                    return if let Some(n_call) = option_yn.take_n_call() {
                        ok_action(Action::OptionYNTuiCallback(n_call))
                    } else {
                        unreachable!("选项必须设定n_callback")
                    };
                }
                ok_none()
            }
            Edit(state) => {
                // f1 按下 进入 帮助页面
                if key_event.is_f1() {
                    return ok_action(Action::ScreenIntent(ToHelp));
                }

                // 如果当前不为 notes编辑，则可响应 up/ down 按键上下
                if state.current_editing_type() != Editing::Notes {
                    // 上移
                    if key_event.is_up() {
                        state.cursor_up();
                        return ok_none();
                    }
                    // 下移
                    if key_event.is_down() {
                        state.cursor_down();
                        return ok_none();
                    }
                }

                // 下移，即使为notes，也应响应tab指令，不然就出不去当前输入框了...
                if key_event.is_tab() {
                    state.cursor_down();
                    return ok_none();
                }
                // 保存
                if key_event.is_ctrl_char('s') {
                    return if state.current_input_validate() {
                        let e_id = state.current_e_id();
                        // 该处已修改：该处不加密，只有 save tip 页面 按下 y 才触发 加密并保存
                        let input_entry = state.current_input_entry();
                        ok_action(Action::ScreenIntent(ToSaveYNOption(input_entry, e_id)))
                    } else {
                        // 验证 to do 未通过验证应给予提示
                        ok_action(Action::SetTuiHotMsg(" Some field is required".into(), Some(3), Some(Alignment::Center)))
                    }
                }
                // 编辑窗口变化
                // 不为 desc 的 响应 enter 到下一行
                if Editing::Notes != state.current_editing_type() {
                    if let KeyCode::Enter = key_event.code {
                        state.cursor_down();
                        return ok_none();
                    }
                }
                // do editing...
                let _ = state.current_editing_string_mut().input(key_event);
                ok_none()
            }
            // 需要主密码
            InputMainPwd(state) => {
                if key_event.is_enter() {
                    return if let Some(security_context) = state.try_build_security_context()? {
                        ok_action(Action::MainPwdVerifySuccess(security_context))
                    } else {
                        state.increment_retry_count()?;
                        ok_none()
                    }
                }
                // 密码编辑窗口变化
                match key_event.code {
                    KeyCode::Backspace => {
                        state.mp_input.pop();
                    }
                    KeyCode::Char(value) => state.mp_input.push(value),
                    _ => {}
                }
               ok_none()
            }
        }
    }
    
}
