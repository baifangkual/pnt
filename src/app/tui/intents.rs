use crate::app::tui::new_dashboard_screen;
use crate::app::tui::rt::TUIApp;
use crate::app::tui::screen::options::OptionYN;
use crate::app::tui::screen::states::{EditingState, NeedMainPwdState};
use crate::app::tui::screen::Screen;
use crate::app::tui::screen::Screen::{DeleteTip, Details, Edit, NeedMainPasswd};
use anyhow::Context;

/// 进入屏幕的意图
/// 该实体的出现是为了修复部分屏幕需显示已解密实体，但还未校验主密码
/// 导致SecurityContext还未生成，从而无法描述要进入的页面的问题
/// 这些行为变体只携带 entryId
#[derive(Debug, Clone)]
pub enum EnterScreenIntent {
    ToDashBoard,
    ToDetail(u32),
    ToEditing(Option<u32>), // 有id为更新，无id为编辑
    ToDeleteTip(u32),
}

impl EnterScreenIntent {
    pub fn handle_intent(&self, tui: &TUIApp) -> anyhow::Result<Screen> {
        if tui.pnt.is_verified() {
            // 已有securityContext，直接发送进入事件
            match &self {
                EnterScreenIntent::ToDetail(e_id) => {
                    let encrypted_entry = tui.pnt.storage.select_entry_by_id(*e_id).context("not found entry")?;
                    let entry = encrypted_entry.decrypt(tui.pnt.try_encrypter()?)?;
                    Ok(Details(entry))
                },
                // 有id为编辑页面
                EnterScreenIntent::ToEditing(Some(e_id)) => {
                    let encrypted_entry = tui.pnt.storage.select_entry_by_id(*e_id).context("not found entry")?;
                    let entry = encrypted_entry.decrypt(tui.pnt.try_encrypter()?)?;
                    Ok(Edit(EditingState::new_updating(entry, *e_id)))
                },
                EnterScreenIntent::ToEditing(None) => {
                    Ok(Edit(EditingState::new_creating()))
                },
                EnterScreenIntent::ToDeleteTip(e_id) => {
                    let encrypted_entry = tui.pnt.storage.select_entry_by_id(*e_id).context("not found entry")?;
                    Ok(DeleteTip(OptionYN::new_delete_tip(encrypted_entry)))
                },
                EnterScreenIntent::ToDashBoard => {
                    Ok(new_dashboard_screen(&tui.pnt))
                }
            }
        } else {
            // 未有主密码则进入需要密码的页面
            Ok(NeedMainPasswd(NeedMainPwdState::new(self.clone())))
        }


    }
}