//! 组件，构成tui元素，能够响应事件

use crate::app::context::PntContext;
use crate::app::entry::InputEntry;
use crate::app::tui::TUIApp;
use crate::app::tui::components::states::{Editing, EditingState, HomePageState, VerifyMPHState};
use crate::app::tui::components::yn::YNState;
use crate::app::tui::events::Action;
use crate::app::tui::intents::ScreenIntent;
use crate::app::tui::intents::ScreenIntent::{ToDeleteYNOption, ToDetail, ToEditing, ToHelp, ToSaveYNOption};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use ratatui::layout::Alignment;
use ratatui::widgets::ListState;

pub(crate) mod states;
pub(crate) mod yn;

/// 当前屏幕
pub enum Screen {
    /// 当前光标指向哪个，因为可能一个元素都没有，所以为 option, 所有元素在entries中
    HomePageV1(HomePageState),
    /// f1 help, list state 为行光标状态
    Help(ListState),
    /// 某详情, u32 为 id
    Details(InputEntry, u32),
    /// 编辑窗口
    Edit(EditingState),
    /// y/n 弹窗
    YNOption(YNState),
    /// 要求键入主密码的窗口，载荷主密码输入string和准备进入的页面
    InputMainPwd(VerifyMPHState),
}

impl Screen {
    /// 表达该屏幕是否为最上级的home_page
    ///
    /// > 该方法给多个可能实现的 home_page 做准备
    pub fn is_home_page(&self) -> bool {
        matches!(self, Screen::HomePageV1(..))
    }

    /// 新建编辑页面
    pub fn new_edit_updating(u_input: InputEntry, e_id: u32) -> Self {
        Screen::Edit(EditingState::new_updating(u_input, e_id))
    }
    /// 新建新建页面
    pub fn new_edit_creating() -> Self {
        Screen::Edit(EditingState::new_creating())
    }

    /// 新建help页面
    pub fn new_help() -> Self {
        Screen::Help(ListState::default())
    }

    /// 新建主页
    pub fn new_home_page1(context: &PntContext) -> Self {
        let vec = context.storage.select_all_entry();
        Screen::HomePageV1(HomePageState::new(vec))
    }

    /// 新建输入密码页面
    pub fn new_input_main_pwd(screen_intent: ScreenIntent, context: &PntContext) -> anyhow::Result<Self> {
        VerifyMPHState::new(screen_intent, context).map(Screen::InputMainPwd)
    }
}

pub trait KeyEventExt {
    /// 判定是否为某char按下
    ///
    /// 注意，该方法仅判定当前code是否为指定char，不判定 modifiers 是否有 shift 按下
    fn is_char(&self, char: char) -> bool;
    /// 判定是否为按下 ctrl 同时 按下某键
    fn is_ctrl_char(&self, char: char) -> bool;
    /// F1
    fn is_f1(&self) -> bool;
    /// tab
    fn is_tab(&self) -> bool;
    /// 上-键盘上
    fn is_up(&self) -> bool;
    /// 下-键盘下
    fn is_down(&self) -> bool;
    /// enter
    fn is_enter(&self) -> bool;
    /// ESC
    fn is_esc(&self) -> bool;
}

impl KeyEventExt for KeyEvent {
    #[inline]
    fn is_char(&self, char: char) -> bool {
        self.code == KeyCode::Char(char)
    }
    #[inline]
    fn is_ctrl_char(&self, char: char) -> bool {
        self.modifiers == KeyModifiers::CONTROL && self.code == KeyCode::Char(char)
    }
    #[inline]
    fn is_f1(&self) -> bool {
        self.code == KeyCode::F(1)
    }
    #[inline]
    fn is_tab(&self) -> bool {
        self.code == KeyCode::Tab
    }
    #[inline]
    fn is_up(&self) -> bool {
        self.code == KeyCode::Up
    }
    #[inline]
    fn is_down(&self) -> bool {
        self.code == KeyCode::Down
    }
    #[inline]
    fn is_enter(&self) -> bool {
        self.code == KeyCode::Enter
    }
    #[inline]
    fn is_esc(&self) -> bool {
        self.code == KeyCode::Esc
    }
}

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

impl EventHandler for TUIApp {
    /// 按键事件处理，需注意，大写不一定表示按下shift，因为还有 caps Lock 键,
    /// 进入该方法的 keyEvent.kind 一定为 按下 KeyEventKind::Press
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Action>> {
        // 每次操作将闲置tick计数清零
        self.idle_tick.reset_idle_tick_count();

        // 任何页面按 ctrl + c 都退出
        if key_event.is_ctrl_char('c') {
            return ok_action(Action::Quit);
        }
        // 按下 esc 的事件，将当前屏幕返回上一个屏幕，若当前为最后一个屏幕，则发送quit事件
        // 若为在find模式的homepage，则退出find...
        if key_event.is_esc() {
            if let Screen::HomePageV1(state) = &mut self.screen {
                if state.find_mode() {
                    return ok_action(Action::TurnOffFindMode);
                } else if !state.current_find_input_is_empty() {
                    state.clear_find_input();
                    return ok_action(Action::FlashVecItems(None));
                } else {
                    self.back_screen();
                }
            } else {
                self.back_screen();
            }
            return ok_none();
        }

        // 委托至 screen 的 key event handler
        self.screen.handle_key_press_event(key_event)
    }
}

impl EventHandler for Screen {
    fn handle_key_press_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Action>> {
        match self {
            // help 页面
            Screen::Help(list_cursor) => {
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
                ok_none()
            }
            // 仪表盘
            Screen::HomePageV1(state) => {
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
                        return ok_action(Action::Relock);
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
                            let _ = state.find_input().input(key_event);
                            ok_none()
                        }
                    }
                }
            }
            // 详情页
            Screen::Details(_, e_id) => {
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
                    return ok_action(Action::Relock);
                }
                if key_event.is_char('e') {
                    let back_and_enter_editing = // 按下e时退出当前屏幕并进入编辑屏幕
                        Action::Actions(vec![Action::BackScreen, Action::ScreenIntent(ToEditing(Some(*e_id)))]);
                    return ok_action(back_and_enter_editing);
                }
                ok_none()
            }
            // 弹窗页面
            Screen::YNOption(option_yn) => {
                if key_event.is_char('q') {
                    return ok_action(Action::BackScreen);
                }
                // 上移
                if key_event.is_char('k') || key_event.is_up() {
                    option_yn.scroll_up();
                    return ok_none();
                }
                // 下移
                if key_event.is_down() || key_event.is_char('j') {
                    option_yn.scroll_down();
                    return ok_none();
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
            Screen::Edit(state) => {
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
                    // 不为 notes 的 响应 enter 到下一行
                    if let KeyCode::Enter = key_event.code {
                        state.cursor_down();
                        return ok_none();
                    }
                }
                // 若当前为编辑notes，且notes内光标在第一行，且按了上键，则选中上方的输入框
                if Editing::Notes == state.current_editing_type()
                    && key_event.is_up()
                    && state.current_editing_string_mut().cursor().0 == 0
                {
                    state.cursor_up();
                    return ok_none();
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
                        ok_action(Action::SetTuiHotMsg(
                            " Some field is required".into(),
                            Some(3),
                            Some(Alignment::Center),
                        ))
                    };
                }
                // 编辑窗口变化
                let _ = state.current_editing_string_mut().input(key_event);
                ok_none()
            }
            // 需要主密码
            Screen::InputMainPwd(state) => {
                if key_event.is_enter() {
                    return if let Some(security_context) = state.try_build_security_context()? {
                        ok_action(Action::MainPwdVerifySuccess(security_context))
                    } else {
                        state.increment_retry_count()?;
                        ok_none()
                    };
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
