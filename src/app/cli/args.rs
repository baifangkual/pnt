use crate::app::runtime::RunMode;
use clap::Parser;

/// lib 运行时给的命令行参数
#[derive(Parser, Debug)]
#[command(version,
about = format!("{}\n  Press F1 in the TUI interface to view the key mappings", clap::crate_description!()),
long_about = None)]
pub struct CliArgs {
    /// (Cli Mode) find entry by name like
    #[arg(short, long)]
    pub find: Option<String>,
}

impl CliArgs {
    pub fn run_mode(&self) -> RunMode {
        // 计算是否使用Cli模式, 一般情况下，若无任意给定的运行时参数，则使用tui，否则cli
        if self.find.is_some() {
            RunMode::Cli
        } else {
            RunMode::Tui
        }
    }
}
