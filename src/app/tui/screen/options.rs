use crate::app::entry::EncryptedEntry;
use crate::app::tui::event::AppEvent;
use crate::app::tui::rt::TUIApp;
use anyhow::anyhow;

/// 二分类枚举
#[derive(Debug, Clone, Copy)]
pub enum YN {
    Yes,
    No,
}

/// 闭包，表示在Y/N情况下的行为
type FnCallYN = Box<dyn Fn(&mut TUIApp) -> anyhow::Result<()> + Send>;

/// 带 YN 选项的实体，可载荷 Item
pub struct YNState<C> {
    pub title: String,
    pub desc: String,
    pub content: Option<C>,
    /// y 状态时触发，可选设定
    pub y_call: Option<FnCallYN>,
    /// n 状态时触发，可选设定
    pub n_call: Option<FnCallYN>,
    /// yn 状态，None表示未设定yn
    pub yn: Option<YN>,
}

impl<C> YNState<C> {
    pub fn content(&self) -> Result<&C, anyhow::Error> {
        match self.content {
            Some(ref c) => Ok(c),
            None => Err(anyhow!("Content not found")),
        }
    }

    pub fn new_just_title_desc(title: &str, desc: &str) -> Self {
        YNState {
            title: title.into(),
            desc: desc.into(),
            content: None,
            y_call: None,
            n_call: None,
            yn: None,
        }
    }
    pub fn change_yn(&mut self, yn: YN) {
        self.yn = Some(yn);
    }
    pub fn set_content(&mut self, c: C) {
        self.content = Some(c);
    }
    pub fn set_y_call(&mut self, call: FnCallYN) {
        self.y_call = Some(call);
    }
    pub fn set_n_call(&mut self, call: FnCallYN) {
        self.n_call = Some(call);
    }

    /// 在 YN set 后进行调用，执行YN对应的行为，
    /// 当 YN 未 set，则返回 Err
    ///
    /// 若对应的YN FnCall 返回Err，则该方法返回对应Err，否则Ok，
    /// 若 YN对应的 FnCall 未有，则Ok
    pub fn call(&self, tui_app: &mut TUIApp) -> anyhow::Result<()> {
        if let Some(YN::Yes) = &self.yn {
            self.call_y(tui_app)
        } else if let Some(YN::No) = &self.yn {
            self.call_n(tui_app)
        } else {
            Err(anyhow!("YN not set"))
        }
    }

    pub fn call_y(&self, tui_app: &mut TUIApp) -> anyhow::Result<()> {
        if let Some(y_call) = &self.y_call {
            y_call(tui_app)
        } else {
            Ok(())
        }
    }

    pub fn call_n(&self, tui_app: &mut TUIApp) -> anyhow::Result<()> {
        if let Some(n_call) = &self.n_call {
            n_call(tui_app)
        } else {
            Ok(())
        }
    }

    pub fn take_y_call(&mut self) -> Option<FnCallYN> {
        self.y_call.take()
    }
    pub fn take_n_call(&mut self) -> Option<FnCallYN> {
        self.n_call.take()
    }
}

impl YNState<EncryptedEntry> {
    /// 删除页面用的
    pub fn new_delete_tip(encrypted_entry: EncryptedEntry) -> Self {
        let d_name = format!("DELETE '{}' ?", &encrypted_entry.name);
        let d_desc = encrypted_entry.description.as_ref().map_or("_", |v| v);
        let e_id = encrypted_entry.id;
        let mut yn = Self::new_just_title_desc(&d_name, &d_desc);
        yn.set_content(encrypted_entry);
        yn.set_y_call(Box::new(move |tui| {
            // 发送删除事件
            tui.send_app_event(AppEvent::EntryRemove(e_id));
            // 响应该事件时 ，当前页面一定为 tips，所以回退到上一级页面（即召唤delete tips页面的页面)
            while !tui.screen.is_dashboard() {
                tui.back_screen();
            }
            Ok(())
        }));
        yn.set_n_call(Box::new(move |tui| Ok(tui.back_screen())));
        yn
    }
}
