use crate::app::context::PntContext;
use crate::app::context::RunMode;
use clap::Parser;
use crate::app::await_verifier_main_pwd;

/// cli 运行 模式
pub fn cli_run(pnt: PntContext) -> anyhow::Result<()> {

    // cli 情况下的要求立即密码
    // 若配置立即要求输入密码则直接校验
    let pnt = if pnt.cfg.need_main_passwd_on_run {
        await_verifier_main_pwd(pnt)?
    } else { pnt };
    
    if let Some(f) = pnt.cli_args.find {
        // to do find Impl
        let vec = pnt.storage.select_entry_by_about_like(&f);
        vec.into_iter()
            .enumerate()
            .for_each(|(i, entry)| println!("{:3}: {}", i + 1, entry.about));
    }
    Ok(())
}



/// lib 运行时给的命令行参数
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// (Cli Mode) find entry by name like
    #[arg(short, long)]
    pub find: Option<String>,
}

impl CliArgs {
    /// 判定运行模式，一般情况下，若无任意给定的运行时参数，则使用tui，否则cli
    pub fn check_run_mode(&self) -> RunMode {
        if self.find.is_some() {
            RunMode::Cli
        } else {
            RunMode::Tui
        }
    }
}
