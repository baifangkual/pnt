use crate::app::crypto::Encrypter;
use crate::app::entry::{EncryptedEntry, InputEntry, ValidEntry};
use crate::app::tui::colors::{CL_BLACK, CL_DARK_DARK_DARK_RED, CL_DARK_DARK_RED, CL_DARK_RED, CL_LIGHT_BLACK, CL_WHITE};
use crate::app::tui::event::AppEvent;
use crate::app::tui::rt::TUIApp;
use ratatui::prelude::Color;

/// 二分类枚举
#[derive(Debug, Clone, Copy)]
pub enum YN {
    Yes,
    No,
}

const THEME_DELETE: Theme = Theme {
    cl_global_bg: CL_DARK_DARK_RED,
    cl_desc_bg: CL_DARK_DARK_DARK_RED,
    cl_title_bg: CL_DARK_RED,
    cl_title_fg: CL_WHITE,
    cl_n_bg: CL_DARK_RED,
    cl_n_fg: CL_WHITE,
    cl_y_bg: CL_DARK_RED,
    cl_y_fg: CL_WHITE,
    cl_desc_fg: CL_WHITE,
};

const THEME_SAVE: Theme = Theme {
    cl_global_bg: CL_LIGHT_BLACK,
    cl_desc_bg: CL_BLACK,
    cl_title_bg: CL_BLACK,
    cl_title_fg: CL_WHITE,
    cl_n_bg: CL_BLACK,
    cl_n_fg: CL_WHITE,
    cl_y_bg: CL_BLACK,
    cl_y_fg: CL_WHITE,
    cl_desc_fg: CL_WHITE,
};

#[derive(Copy, Clone)]
pub struct Theme {
    /// bg yn页面全局
    pub cl_global_bg: Color,
    /// bg desc部分
    pub cl_desc_fg: Color,
    /// fg desc文字颜色
    pub cl_desc_bg: Color,
    /// fg title文字颜色
    pub cl_title_fg: Color,
    /// bg title
    pub cl_title_bg: Color,
    /// y选项fy
    pub cl_y_fg: Color,
    /// y选项bg
    pub cl_y_bg: Color,
    /// n选项fy
    pub cl_n_fg: Color,
    /// n选项bg
    pub cl_n_bg: Color,
}

/// 闭包，表示在Y/N情况下的行为
type FnCallYN = Box<dyn FnOnce(&mut TUIApp) -> anyhow::Result<()> + Send>;

/// 带 YN 选项的实体，可载荷 Item
pub struct YNState {
    pub title: String,
    pub desc: String,
    /// y 状态时触发，可选设定
    pub y_call: Option<FnCallYN>,
    /// n 状态时触发，可选设定
    pub n_call: Option<FnCallYN>,
    /// yn 状态，None表示未设定yn
    pub yn: Option<YN>,
    /// 显示颜色信息
    pub theme: Theme,
}

impl YNState {
    pub fn new(title: String, desc: String, theme: Theme) -> Self {
        YNState {
            title,
            desc,
            y_call: None,
            n_call: None,
            yn: None,
            theme,
        }
    }

    pub fn theme_mut(&mut self) -> &mut Theme {
        &mut self.theme
    }
    pub fn change_yn(&mut self, yn: YN) {
        self.yn = Some(yn);
    }
    pub fn set_y_call(&mut self, call: FnCallYN) {
        self.y_call = Some(call);
    }
    pub fn set_n_call(&mut self, call: FnCallYN) {
        self.n_call = Some(call);
    }

    pub fn take_y_call(&mut self) -> Option<FnCallYN> {
        self.y_call.take()
    }
    pub fn take_n_call(&mut self) -> Option<FnCallYN> {
        self.n_call.take()
    }
}

impl YNState {
    /// 删除页面用的
    pub fn new_delete_tip(encrypted_entry: EncryptedEntry) -> Self {
        let e_name = &encrypted_entry.about;
        let e_desc = encrypted_entry.notes.as_ref().map_or("_", |v| v);
        let tip_title = format!("DELETE '{}' ?", e_name);
        let tip_desc = format!(
            "[󰦨 about]: {}\n\
             -󰦨 notes-------------\n{}",
            e_name, e_desc
        );
        let e_id = encrypted_entry.id;
        let mut yn = Self::new(tip_title, tip_desc, THEME_DELETE);
        yn.set_y_call(Box::new(move |tui| {
            // 发送删除事件
            tui.send_app_event(AppEvent::EntryRemove(e_id));
            // 响应该事件时 ，当前页面一定为 tips，所以回退到上一级页面（即召唤delete tips页面的页面)
            while !tui.screen.is_dashboard() {
                tui.back_screen();
            }
            Ok(())
        }));
        yn.set_n_call_back_screen();
        yn
    }
    /// 保存页面用的
    pub fn new_save_tip(ie: InputEntry, e_id: Option<u32>) -> Self {
        let e_notes_dots = if ie.notes.is_empty() { "_" } else { &ie.notes };
        let tip_title = if e_id.is_none() {
            format!("SAVE '{}' ?", ie.about)
        } else {
            format!("SAVE CHANGE '{}' ?", ie.about)
        };
        let tip_desc = format!(
            "[󰦨 about]:    {}\n\
             [󰌿 username]: {}\n\
             [󰌿 password]: {}\n\
             -󰦨 notes-------------\n{}",
            &ie.about, &ie.username, &ie.password, e_notes_dots
        );
        let mut yn = Self::new(tip_title, tip_desc, THEME_SAVE);
        yn.set_y_call(Box::new(move |tui| {
            let valid = tui.pnt.try_encrypter()?.encrypt(&ie)?;
            if let Some(e_id) = e_id {
                tui.send_app_event(AppEvent::EntryUpdate(valid, e_id))
            } else {
                tui.send_app_event(AppEvent::EntryInsert(valid));
            }
            // 响应该事件时 ，当前页面一定为 tips，所以回退到上一级页面（即召唤delete tips页面的页面)
            while !tui.screen.is_dashboard() {
                tui.back_screen();
            }
            Ok(())
        }));
        yn.set_n_call_back_screen();
        yn
    }

    fn set_n_call_back_screen(&mut self) {
        self.set_n_call(Box::new(move |tui| Ok(tui.back_screen())))
    }
}
