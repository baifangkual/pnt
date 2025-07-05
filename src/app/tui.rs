mod util;
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
    debug!("start run TUI mode");
    let terminal = ratatui::init();
    let running = TUIRuntime::with_pnt(pnt, (NoEncrypter, NoEncrypter));
    let result = running.run(terminal);
    ratatui::restore();
    result
}

