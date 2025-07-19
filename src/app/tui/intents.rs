use crate::app::entry::InputEntry;
use crate::app::tui::TUIApp;
use crate::app::tui::components::Screen;
use crate::app::tui::components::Screen::{Details, Edit, YNOption};
use crate::app::tui::components::states::EditingState;
use crate::app::tui::components::yn::YNState;
use anyhow::Context;

/// 进入屏幕的意图
/// 该实体的出现是为了修复部分屏幕需显示已解密实体，但还未校验主密码
/// 导致SecurityContext还未生成，从而无法描述要进入的页面的问题
/// 这些行为变体只携带 entryId
#[derive(Debug, Clone)]
pub enum ScreenIntent {
    ToHelp,
    ToHomePageV1,
    ToDetail(u32),
    ToEditing(Option<u32>), // 有id为更新，无id为编辑
    ToDeleteYNOption(u32),
    ToSaveYNOption(InputEntry, Option<u32>), // 保存提示页面
}

impl ScreenIntent {
    /// 表达该 屏幕 在进入前是否需要 主密码
    pub fn is_before_enter_need_main_pwd(&self) -> bool {
        match self {
            ScreenIntent::ToHelp | ScreenIntent::ToHomePageV1 => false,
            _ => true,
        }
    }
}

impl ScreenIntent {
    pub fn handle_intent(&self, tui: &TUIApp) -> anyhow::Result<Screen> {
        if !tui.context.is_verified() && self.is_before_enter_need_main_pwd() {
            // 未有主密码则进入需要密码的页面
            Screen::new_input_main_pwd(self.clone(), &tui.context)
        } else {
            // 已有securityContext，直接发送进入事件
            match &self {
                ScreenIntent::ToDetail(e_id) => {
                    let encrypted_entry = tui
                        .context
                        .storage
                        .select_entry_by_id(*e_id)
                        .context("not found entry")?;
                    let entry = encrypted_entry.decrypt(tui.context.try_encrypter()?)?;
                    Ok(Details(entry, *e_id))
                }
                // 有id为编辑页面
                ScreenIntent::ToEditing(Some(e_id)) => {
                    let encrypted_entry = tui
                        .context
                        .storage
                        .select_entry_by_id(*e_id)
                        .context("not found entry")?;
                    let entry = encrypted_entry.decrypt(tui.context.try_encrypter()?)?;
                    Ok(Edit(EditingState::new_updating(entry, *e_id)))
                }
                ScreenIntent::ToEditing(None) => Ok(Edit(EditingState::new_creating())),
                ScreenIntent::ToDeleteYNOption(e_id) => {
                    let encrypted_entry = tui
                        .context
                        .storage
                        .select_entry_by_id(*e_id)
                        .context("not found entry")?;
                    Ok(YNOption(YNState::new_delete_tip(encrypted_entry)))
                }
                ScreenIntent::ToSaveYNOption(ve, e_id) => Ok(YNOption(YNState::new_save_tip(ve.clone(), *e_id))),
                ScreenIntent::ToHomePageV1 => Ok(Screen::new_home_page1(&tui.context)),
                ScreenIntent::ToHelp => Ok(Screen::new_help()),
            }
        }
    }
}
