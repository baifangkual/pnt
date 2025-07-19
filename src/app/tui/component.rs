//! 组件，构成tui元素，能够响应事件

use anyhow::anyhow;
use crate::app::tui::event::Action;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Alignment;
use crate::app::crypto::build_mpv;
use crate::app::tui::event::key_ext::KeyEventExt;
use crate::app::tui::intents::ScreenIntent::{ToDeleteYNOption, ToDetail, ToEditing, ToHelp, ToSaveYNOption};
use crate::app::tui::screen::Screen;
use crate::app::tui::screen::Screen::{Details, Edit, Help, HomePageV1, NeedMainPasswd, YNOption};
use crate::app::tui::screen::states::Editing;

pub trait Component {
    /// 响应 key 按下事件，要求self的可变引用，即对事件的响应可能改变自身状态，
    /// 方法返回 Option<Action>，即响应可能产出对应行为要求
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> Option<Action>;
}

impl Component for Screen {
    
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> Option<Action> {
        match self {
            // help 页面
            Help(state) => {
                if key_event.is_char('q') {
                    return Some(Action::BackScreen);
                }
                // 上移
                if key_event.is_char('k') || key_event.is_up() {
                    state.select_previous();
                    return None;
                }
                // 下移
                if key_event.is_down() || key_event.is_char('j') {
                    state.select_next();
                    return None;
                }
                // 顶部 底部
                if let KeyCode::Char('g') | KeyCode::Home = key_event.code {
                    state.select_first();
                    return None;
                }
                // 顶部 底部
                if let KeyCode::Char('G') | KeyCode::End = key_event.code {
                    state.select_last();
                    return None;
                }
                return None;
            }
            // 仪表盘
            HomePageV1(state) => {
                // f1 按下 进入 帮助页面
                if key_event.is_f1() {
                    return Some(Action::ScreenIntent(ToHelp));
                }

                // home_page find
                if !state.find_mode() {
                    if key_event.is_char('f') {
                        return Some(Action::TurnOnFindMode);
                    }
                    // 响应 按下 l 丢弃 securityContext以重新锁定
                    if key_event.is_char('l') {
                        return Some(Action::RELOCK);
                    }
                    if key_event.is_char('q') {
                        return Some(Action::BackScreen);
                    }
                    // 可进入 查看，编辑，删除tip，新建 页面
                    // 若当前光标无所指，则只能 创建
                    if let Some(c_ptr) = state.cursor_selected() {
                        let curr_ptr_e_id = state.entries()[c_ptr].id;
                        // open
                        if key_event.is_char('o') || key_event.is_enter() {
                            return Some(Action::ScreenIntent(ToDetail(curr_ptr_e_id)));
                        }
                        // edit
                        if key_event.is_char('e') {
                            return Some(Action::ScreenIntent(ToEditing(Some(curr_ptr_e_id))));
                        }
                        // delete 但是home_page 的光标？
                        // 任何删除都应确保删除页面上一级为home_page
                        // 即非home_page接收到删除事件时应确保关闭当前并打开删除
                        if key_event.is_char('d') {
                            return Some(Action::ScreenIntent(ToDeleteYNOption(curr_ptr_e_id)));
                        }
                        // 上移
                        if key_event.is_char('k') || key_event.is_up() {
                            state.cursor_up();
                            return None;
                        }
                        // 下移
                        if key_event.is_down() || key_event.is_char('j') {
                            state.cursor_down();
                            return None;
                        }
                        // 顶部 底部
                        if let KeyCode::Char('g') | KeyCode::Home = key_event.code {
                            state.cursor_mut_ref().select_first();
                            return None;
                        }
                        // 顶部 底部
                        if let KeyCode::Char('G') | KeyCode::End = key_event.code {
                            state.cursor_mut_ref().select_last();
                            return None;
                        }
                    }
                    // 任意光标位置都可以新建
                    if key_event.is_char('a') {
                        return Some(Action::ScreenIntent(ToEditing(None)));
                    }
                    None
                } else {
                    match key_event.code {
                        KeyCode::Enter => Some(Action::TurnOffFindMode),
                        _ => {
                            // 返回bool表示是否修改了，暂时用不到
                            let _ = state.find_textarea().input(key_event);
                            None
                        }
                    }
                }
            }
            // 详情页
            Details(_, e_id) => {
                // f1 按下 进入 帮助页面
                if key_event.is_f1() {
                    return Some(Action::ScreenIntent(ToHelp));
                }

                if key_event.is_char('q') {
                    return Some(Action::BackScreen);
                }
                if key_event.is_char('d') {
                    let de_id = *e_id;
                    return Some(Action::ScreenIntent(ToDeleteYNOption(de_id)));
                }
                if key_event.is_char('l') {
                  return Some(Action::RELOCK);
                }
                None
            }
            // 弹窗页面
            YNOption(option_yn) => {
                if key_event.is_char('q') {
                    return Some(Action::BackScreen);
                }
                if let KeyCode::Char('y' | 'Y') | KeyCode::Enter = key_event.code {
                    return if let Some(y_call) = option_yn.take_y_call() {
                        Some(Action::OptionYNTuiCallback(y_call))
                    } else {
                        unreachable!("选项必须设定y_callback")
                    };
                }
                if let KeyCode::Char('n' | 'N') = key_event.code {
                    return if let Some(n_call) = option_yn.take_n_call() {
                        Some(Action::OptionYNTuiCallback(n_call))
                    } else {
                        unreachable!("选项必须设定n_callback")
                    };
                }
                None
            }
            Edit(state) => {
                // f1 按下 进入 帮助页面
                if key_event.is_f1() {
                    return Some(Action::ScreenIntent(ToHelp));
                }

                // 如果当前不为 notes编辑，则可响应 up/ down 按键上下
                if state.current_editing_type() != Editing::Notes {
                    // 上移
                    if key_event.is_up() {
                        state.cursor_up();
                        return None;
                    }
                    // 下移
                    if key_event.is_down() {
                        state.cursor_down();
                        return None;
                    }
                }

                // 下移，即使为notes，也应响应tab指令，不然就出不去当前输入框了...
                if key_event.is_tab() {
                    state.cursor_down();
                    return None;
                }
                // 保存
                if key_event.is_ctrl_char('s') {
                    return if state.current_input_validate() {
                        let e_id = state.current_e_id();
                        // 该处已修改：该处不加密，只有 save tip 页面 按下 y 才触发 加密并保存
                        let input_entry = state.current_input_entry();
                        Some(Action::ScreenIntent(ToSaveYNOption(input_entry, e_id)))
                    } else {
                        // 验证 to do 未通过验证应给予提示
                        Some(Action::SetTuiHotMsg(" Some field is required".into(), Some(3), Some(Alignment::Center)))
                    }
                }
                // 编辑窗口变化
                // 不为 desc 的 响应 enter 到下一行
                if Editing::Notes != state.current_editing_type() {
                    if let KeyCode::Enter = key_event.code {
                        state.cursor_down();
                        return None;
                    }
                }
                // do editing...
                let _ = state.current_editing_string_mut().input(key_event);
                None
            }
            // 需要主密码
            NeedMainPasswd(state) => {
                if key_event.is_enter() {
                    if let Some(security_context) = state.try_build_security_context() {
                        return Some(Action::MainPwdVerifySuccess(security_context))
                    } else { 
                        return Some(Action::MainPwdVerifyFailed)
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
               None
            }
        }
    }
    
}
