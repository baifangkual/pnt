use crate::app::tui::new_dashboard_screen;
use crate::app::tui::rt::TUIApp;
use crate::app::tui::screen::Screen;
use crate::app::tui::screen::Screen::{YNOption, Details, Edit, Help, NeedMainPasswd};
use crate::app::tui::screen::yn::YNState;
use crate::app::tui::screen::states::{EditingState, NeedMainPwdState};
use anyhow::Context;
use crate::app::entry::ValidEntry;

/// 进入屏幕的意图
/// 该实体的出现是为了修复部分屏幕需显示已解密实体，但还未校验主密码
/// 导致SecurityContext还未生成，从而无法描述要进入的页面的问题
/// 这些行为变体只携带 entryId
#[derive(Debug, Clone)]
pub enum EnterScreenIntent {
    ToHelp,
    ToDashBoardV1,
    ToDetail(u32),
    ToEditing(Option<u32>), // 有id为更新，无id为编辑
    ToDeleteYNOption(u32),
    ToSaveYNOption(ValidEntry, Option<u32>), // 保存提示页面
}

impl EnterScreenIntent {
    /// 表达该 屏幕 在进入前是否需要 主密码
    pub fn is_before_enter_need_main_pwd(&self) -> bool {
        match self {
            EnterScreenIntent::ToHelp | EnterScreenIntent::ToDashBoardV1 => false,
            _ => true,
        }
    }
}

impl EnterScreenIntent {
    pub fn handle_intent(&self, tui: &TUIApp) -> anyhow::Result<Screen> {
        if !tui.pnt.is_verified() && self.is_before_enter_need_main_pwd() {
            // 未有主密码则进入需要密码的页面
            Ok(NeedMainPasswd(NeedMainPwdState::new(self.clone())))
        } else {
            // 已有securityContext，直接发送进入事件
            match &self {
                EnterScreenIntent::ToDetail(e_id) => {
                    let encrypted_entry = tui
                        .pnt
                        .storage
                        .select_entry_by_id(*e_id)
                        .context("not found entry")?;
                    let entry = encrypted_entry.decrypt(tui.pnt.try_encrypter()?)?;
                    Ok(Details(entry, *e_id))
                }
                // 有id为编辑页面
                EnterScreenIntent::ToEditing(Some(e_id)) => {
                    let encrypted_entry = tui
                        .pnt
                        .storage
                        .select_entry_by_id(*e_id)
                        .context("not found entry")?;
                    let entry = encrypted_entry.decrypt(tui.pnt.try_encrypter()?)?;
                    Ok(Edit(EditingState::new_updating(entry, *e_id)))
                }
                EnterScreenIntent::ToEditing(None) => Ok(Edit(EditingState::new_creating())),
                EnterScreenIntent::ToDeleteYNOption(e_id) => {
                    let encrypted_entry = tui
                        .pnt
                        .storage
                        .select_entry_by_id(*e_id)
                        .context("not found entry")?;
                    Ok(YNOption(YNState::new_delete_tip(encrypted_entry)))
                },
                EnterScreenIntent::ToSaveYNOption(ve, e_id) => {
                  Ok(YNOption(YNState::new_save_tip(ve.clone(), *e_id)))  
                },
                EnterScreenIntent::ToDashBoardV1 => Ok(new_dashboard_screen(&tui.pnt)),
                EnterScreenIntent::ToHelp => Ok(Help),
            }
        }
    }
}
