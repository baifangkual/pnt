use clap::Parser;

pub mod util;

/// pnt 运行时给的命令行参数
/// 使用 clap 进行命令行命令映射到该结构体
/// 结构体内变量 doc 会作为 --help 输出
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct PntCmdLineArgs {
    /// Name of the person to greet
    #[arg(short, long, required = false)]
    pub name: String,
    
    /// enable debug mode
    #[arg(long, required = false)]
    pub debug: bool,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    pub count: u8,
}

pub fn cmd_args() {
    let version = clap::crate_version!();
    let description = clap::crate_description!();
    let app_name = clap::crate_name!();
    let authors = clap::crate_authors!();
    
}
