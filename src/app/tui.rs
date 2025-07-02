pub mod util;

use crate::app::runtime::PntRuntimeContext;
use log::debug;

/// tui 运行 模式
pub fn tui_run(pnt: PntRuntimeContext) -> anyhow::Result<()> {
    debug!("start run TUI mode");
    // json_edit_exp::run()?;
    Ok(())
}
