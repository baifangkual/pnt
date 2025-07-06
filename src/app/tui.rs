mod layout;
mod ui;
mod event;
mod runtime;
mod screen;
mod widgets;

use crate::app::context::PntContext;
use log::debug;
use crate::app::crypto::NoEncrypter;
use crate::app::tui::runtime::TUIRuntime;

/// tui 运行 模式
pub fn tui_run(pnt: PntContext) -> anyhow::Result<()> {
    let terminal = ratatui::init();
    let running = TUIRuntime::with_pnt(pnt);
    let result = running.run(terminal);
    ratatui::restore();
    result
}

