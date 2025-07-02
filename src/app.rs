mod cli;
mod config;
mod encrypt;
mod entry;
mod runtime;
mod storage;
mod tui;

use crate::app::config::{Cfg, load_cfg};
use crate::app::encrypt::{Encrypter, MainPwdEncrypter, MainPwdVerifier};
use crate::app::runtime::{MAIN_PASS_KEY, NoteState, PntRuntimeContext, RunMode};
use crate::app::storage::sqlite::SqliteConn;
use anyhow::{Context, Error, Result};
use clap::Parser;
use cli::args::CliArgs;
use log::debug;
use ratatui::crossterm::style::Stylize;
use std::io::ErrorKind;

/// 向stdin索要输入的密码，若有utf8字符则提示无效字符
/// 该方法内会loop阻塞直到输入有效字符
/// 该方法内校验输入密码字符长度至少大于等于6
fn read_stdin_passwd() -> Result<String> {
    loop {
        match rpassword::prompt_password("Main password: ".yellow()) {
            Ok(p) => {
                if p.chars().filter(|c| !c.is_ascii_control()).count() < 6 {
                    println!("{}", "> Password too short".red())
                } else {
                    return Ok(p);
                }
            }
            Err(io_e) => match io_e.kind() {
                // 当输入形如utf8字符时 rpassword 的该异常 "stream did not contain valid UTF-8"
                // 转为告知用户无效字符，其他底层系统异常向上返回
                ErrorKind::InvalidData => {
                    println!(
                        "{}",
                        "> Password contains invalid characters, please re-enter".red()
                    )
                }
                _ => return Err(Error::from(io_e)),
            },
        }
    }
}

/// 初始化 db 位置
fn init_storage(cfg: &Cfg) -> Result<SqliteConn> {
    SqliteConn::new(&cfg.date)
        .with_context(|| format!("Failed to init data for path: {}", &cfg.date.display()))
}

/// 阻塞读取stdin，
/// 要求至少两次主密码,
/// 至少要求密码字符大于等于6个
/// 返回的字符串为明文
fn init_main_pwd_by_stdin() -> Result<String> {
    let mut vec = Vec::with_capacity(2);
    let p = loop {
        if vec.is_empty() {
            println!(
                "{}",
                "Init main password: Enter or press CTRL+C to exit".yellow()
            );
        } else {
            println!(
                "{}",
                "Init main password: Enter again or press CTRL+C to exit".yellow()
            );
        }
        // 该并不支持中文，密码字符有所限制，应显式提示
        let rl = read_stdin_passwd()?;
        vec.push(rl);
        debug!("vec len: {}", &vec.len());
        // 判定是否两个且相等
        if vec.len() >= 2 {
            if vec[vec.len() - 1] == vec[vec.len() - 2] {
                // 两次相等，返回
                break vec.pop().unwrap();
            } else {
                println!(
                    "{}",
                    "> Passwords entered twice do not match, please re-enter".red()
                );
                vec.clear();
            }
        }
    };
    Ok(p)
}

/// 使用NoteState校验当前db文件状态，
/// 若不存在则stdin提示要求输入db位置或新建db，
/// 若存在但无main-pwd则要求设定之，
/// 若存在且有main-pwd，则直接返回连接的db的conn
fn pre_note_state_init_check(cfg: &Cfg) -> Result<SqliteConn> {
    // 检查 db 状态
    let mut state = NoteState::check(cfg);
    // conn，下 loop 完应为 Some
    let mut storage: Option<SqliteConn> = None;

    // 初始化校验及获取 sql conn
    loop {
        match state {
            NoteState::NoStorage => {
                storage = Some(init_storage(&cfg)?);
                // stdout print init 位置
                println!(
                    "{}{}",
                    "Init pnt data storage with: ".on_white().black(),
                    &cfg.date.display().to_string().on_white().black()
                );
                state = NoteState::NoMainPwd;
            }
            NoteState::NoMainPwd => {
                // init main pwd
                let mp = init_main_pwd_by_stdin()?;
                let emp = MainPwdEncrypter::from_salt(&cfg.salt).encrypt(mp)?;
                // 从 st中拿（NoStorage创建的）或自己创建
                let mut st = if let None = storage {
                    SqliteConn::new(&cfg.date)
                        .with_context(|| format!("Failed to conn: {}", &cfg.date.display()))?
                } else {
                    storage.take().unwrap()
                };
                st.insert_cfg(MAIN_PASS_KEY, &emp);
                // 用完归还或给其
                storage = Some(st);
                state = NoteState::Ready
            }
            NoteState::Ready => {
                if storage.is_none() {
                    let st = SqliteConn::new(&cfg.date)
                        .with_context(|| format!("Failed to conn: {}", &cfg.date.display()))?;
                    storage = Some(st);
                }
                break;
            }
        }
    }
    // 到达这里不会为None
    Ok(storage.unwrap())
}

/// 等待 stdin输入并校验主密码，当失败到一定次数时
/// 释放sqlite 对文件的连接资源并退出进程
/// 该方法要求SqliteConn所有权，因为内部可能执行 drop
/// 该方法要么返回 （校验器，SqliteConn），要么因stdin错误返回Err，要么退出进程（重试次数到顶）
fn await_verifier_main_pwd(
    cfg: &Cfg,
    storage: SqliteConn,
) -> Result<(MainPwdVerifier, SqliteConn)> {
    let mp_hash_b64d = storage.select_cfg_v_by_key(MAIN_PASS_KEY).unwrap(); // 代码走到当前行该mp一定不为None ...
    let verifier = MainPwdVerifier::from_salt_and_passwd(&cfg.salt, mp_hash_b64d);
    // 后续可设定该值为inner配置项，且重试大于一定次数可选操作... 比如删除库文件？
    let max_loop = 5_u8;
    for n in 0..max_loop {
        let mp = read_stdin_passwd()?;
        if verifier.verify(mp) {
            // 验证通过，返回主校验器
            return Ok((verifier, storage));
        } else {
            // 校验失败，提示
            let tip = format!("{} ({}/{})", "Password is incorrect", n + 1, max_loop);
            println!("{}", tip.on_dark_red().white())
        }
    }
    // 至此，证明for走完仍为校验通过，进程结束
    drop(storage); // 释放sqlite 对文件的连接资源
    std::process::exit(3);
}

/// pnt 程序入口
pub fn pnt_run() -> Result<()> {
    // 前置，对 Cli 参数如 -h -V 等做出响应并退出
    let cli_line = CliArgs::parse();
    // 非 -help 等提前退出的参数则进入流程

    // 载入配置
    let cfg = load_cfg();

    // to do impl cli tui select run
    let storage = pre_note_state_init_check(&cfg)?;
    // 校验配置，是否在运行时需要密码，当前还未要求终端进入 原始模式，遂可以读stdin

    let (mpv_or, conn) = if cfg.need_main_passwd_on_run {
        let (mpv, conn) = await_verifier_main_pwd(&cfg, storage)?;
        (Some(mpv), conn)
    } else {
        (None, storage)
    };

    let run_mode = cli_line.run_mode();
    debug!("run_mode: {:?}", run_mode);
    // to do app init
    let pnt = PntRuntimeContext::new(cfg, cli_line, conn, run_mode, mpv_or);
    // to do app run
    run_with_context(pnt)
}

fn run_with_context(pnt: PntRuntimeContext) -> Result<()> {
    // 库已初始化，验证是否使用 Cli 模式
    if pnt.run_mode == RunMode::Cli {
        cli_run(pnt)
    } else {
        tui_run(pnt)
    }
}

fn cli_run(pnt: PntRuntimeContext) -> Result<()> {
    if let Some(f) = pnt.cli_args.find {
        // todo find Impl
        println!("find: {:?}", f);
    }
    Ok(())
}
fn tui_run(pnt: PntRuntimeContext) -> Result<()> {
    debug!("start run TUI mode");
    // json_edit_exp::run()?;
    Ok(())
}

#[cfg(test)]
mod tests {}
