mod cfg;
mod cli;
mod consts;
mod context;
mod crypto;
mod entry;
mod errors;
mod storage;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::CliArgs;

/// pnt 程序入口
pub fn pnt_run() -> Result<()> {
    // 前置，对 Cli 参数如 -h -V 等做出响应并退出
    let cli = CliArgs::parse();
    // cli 执行，若返回context则要求tui运行
    let Some(context) = cli.run()? else { return Ok(()) };
    tui::tui_run(context)
}
