mod util;
mod ui;
mod event;
mod running;

use crate::app::runtime::PntRuntimeContext;
use log::debug;
use crate::app::tui::running::TUIRunning;

/// tui 运行 模式
pub fn tui_run(pnt: PntRuntimeContext) -> anyhow::Result<()> {
    debug!("start run TUI mode");
    // json_edit_exp::run()?;
    let terminal = ratatui::init();
    let running = TUIRunning::with_pnt(pnt);
    let result = running.run(terminal);
    ratatui::restore();
    result
}

