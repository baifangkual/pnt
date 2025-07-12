use crate::app::cfg::load_cfg;
use crate::app::consts::ALLOC_INVALID_MAIN_PASS_MAX;
use crate::app::context::{DataFileState, PntContext};
use crate::app::crypto::{Encrypter, MainPwdEncrypter};
use crate::app::errors::AppError;
use crate::app::storage::Storage;
use anyhow::anyhow;
use clap::ValueHint;
use clap::{Parser, Subcommand};
use ratatui::crossterm::style::Stylize;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// 子命令定义
#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Initializing pnt data storage location
    Init,
    /// Reset main password
    #[command(name = "rs-mp", long_about = "Reset the main password in an interactive context")]
    ResetMainPwd,
}

/// runtime cli args...
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Find for entries with similar 'about' values
    #[arg(short = 'f', long = "find", value_name = "ABOUT")]
    pub find: Option<String>,
    /// Run with the given pnt data file
    #[arg(short='d', long= "data", value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub data: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

impl CliArgs {
    /// cli 运行，若Err，则应向main返回要求进程非成功退出，
    /// 若Ok(Some(context))则表明要求TUI运行，
    /// 若OK(None) 则表明成功cli运行结束，程序成功退出
    pub fn run(&self) -> anyhow::Result<Option<PntContext>> {
        // 看看参数要求
        if let Some(CliCommand::Init) = &self.command {
            // 显式要求 init
            handle_pnt_data_init()?;
            return Ok(None);
        } else if let Some(CliCommand::ResetMainPwd) = &self.command {
            // todo modifymain pawd
            println!("aaaaaaaaaaaaaaaaaa-------Modify main password in an interactive context");

            return Ok(None);
        }
        let mut cfg = load_cfg()?;
        // 没有给 -data 就连接默认数据文件
        let need_load_data_file = self.data.as_ref().unwrap_or(&cfg.default_date);
        // 连接数据文件，因为为非显式init，所以任何失败情况该方法内均Err向上回报
        let conn = assert_data_file_ready(need_load_data_file)?;
        // 已填充inner配置的cfg
        cfg.overwrite_inner_cfg(&conn)?;
        // pnt 上下文
        let mut context = PntContext::new_with_un_verified(cfg, conn);

        // cli 要求 find
        if let Some(find) = &self.find {
            if context.is_need_mp_on_run() {
                context = await_verifier_main_pwd(context)?;
            }
            context
                .storage
                .select_entry_by_about_like(&find)
                .into_iter()
                .enumerate()
                .for_each(|(i, entry)| println!("{:>3}: {}", i + 1, entry.about));
            // find 后不需要 tui 运行，返回NONE 要求结束进程
            return Ok(None);
        }

        // 返回 Context 要求 tui 运行
        Ok(Some(context))
    }
}

use crate::app::consts;

/// 初始化 pnt 数据文件
///
/// 该方法内将根据env及配置文件等设置情况
fn handle_pnt_data_init() -> anyhow::Result<()> {
    println!("{}", "pnt data file initialized...\n".bold().dark_cyan());
    let data_local_path = if let Some(dp) = crate::app::cfg::env_data_file_path() {
        let msg = format!(
            "find env:[{}='{}']\n",
            consts::ENV_DEFAULT_DATA_FILE_PATH_KEY,
            dp.display()
        );
        println!("{}", msg.grey());
        dp
    } else {
        let msg = format!(
            "not find env:[{}]\ntry find config file...\n",
            consts::ENV_DEFAULT_DATA_FILE_PATH_KEY
        );
        println!("{}", msg.grey());
        let config_path = if let Some(cp) = crate::app::cfg::env_conf_path() {
            let msg = format!("find env:[{}='{}']\n", consts::ENV_CONF_PATH_KEY, cp.display());
            println!("{}", msg.grey());
            cp
        } else {
            let cp = crate::app::cfg::default_conf_path();
            let msg = format!(
                "not find env:[{}],\ntry read config default local with: '{}'\n",
                consts::ENV_CONF_PATH_KEY,
                cp.display().to_string()
            );
            println!("{}", msg.grey());
            cp
        };
        // 尝试从磁盘读取配置文件
        let cfg = crate::app::cfg::try_load_cfg_from_disk(&config_path)?;
        if let Some(toml_cfg) = cfg {
            if let Some(df) = toml_cfg.default_data {
                // 存在配置文件，存在配置
                let msg = format!(
                    "config '{}' exists,\nfind: 'default_data'={}\n",
                    config_path.display(),
                    df.display()
                );
                println!("{}", msg.grey());
                df
            } else {
                let default_data_path = crate::app::cfg::default_data_path();
                // 存在配置文件，但没有该项配置
                let msg = format!(
                    "config '{}' exists, but not set 'default_data',\nwill use default data file path: {}\n",
                    config_path.display(),
                    default_data_path.display()
                );
                println!("{}", msg.grey());
                default_data_path
            }
        } else {
            let default_data_path = crate::app::cfg::default_data_path();
            // none 为文件不存在
            let msg = format!(
                "config '{}' not exists,\nuse default data file path: {}\n",
                config_path.display(),
                default_data_path.display()
            );
            println!("{}", msg.grey());
            default_data_path
        }
    };
    let msg = format!("will create data file with: '{}'", data_local_path.display());
    println!("{}", msg.bold().cyan());
    println!("\npress Enter to init main password with interactive context or press Ctrl-C to exit");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    // 初始化主密码
    let mph = MainPwdEncrypter::new_from_random_salt().encrypt(init_main_pwd_by_stdin()?)?;
    println!("{}", "successfully init main password".green());

    // 检查 data local path 位置是否存在文件，若存在，则提示其是否覆盖
    if data_local_path.exists() {
        println!("\nfile '{}'already exists", data_local_path.display());
        println!("{}", "overwrite this file?".red());
        println!("\nenter 'yes' to overwrite existing file or press Ctrl-C to exit");
        buf.clear();
        std::io::stdin().read_line(&mut buf)?;
        if buf.to_lowercase().trim() == "yes" {
            std::fs::remove_file(&data_local_path)?;
        } else {
            return Err(anyhow!(
                "file '{}' already exists,\ncannot create pnt data file",
                data_local_path.display().to_string()
            ));
        }
    }

    println!("\nmain password hash:\n{mph}\n");
    let mut conn = Storage::open_in_memory()?;
    conn.store_b64_s_mph(&mph);
    // 存储数据文件至指定位置, 该方法不会覆盖文件，位置已有会Err
    conn.db_mem_to_disk(&data_local_path)?;
    let msg = format!("data file created: {}", data_local_path.display());
    println!("{}", msg.bold().cyan());
    println!("{}", "\nsuccessfully created data file".green());
    Ok(())
}

/// 向stdin索要输入的密码，若有utf8字符则提示无效字符
///
/// 若给定check_too_short参数则该方法内校验输入密码字符长度至少大于等于给定参数
///
/// 该方法内会 loop 阻塞当前线程直到输入有效字符返回或收到 Ctrl + C 终止信号停止进程
fn loop_read_stdin_ascii_passwd(check_too_short: Option<u8>) -> anyhow::Result<String> {
    loop {
        match rpassword::prompt_password("Main password: ".yellow()) {
            Ok(p) => {
                if let Some(min) = check_too_short {
                    if p.chars().filter(|c| !c.is_ascii_control()).count() < min as usize {
                        println!("{}", "> Password too short".red());
                        continue;
                    }
                }
                return Ok(p);
            }
            Err(io_e) => match io_e.kind() {
                // 当输入形如utf8字符时 rpassword 的该异常 "stream did not contain valid UTF-8"
                // 转为告知用户无效字符，其他底层系统异常向上返回
                ErrorKind::InvalidData => {
                    println!("{}", "> Password contains invalid characters, please re-enter".red())
                }
                _ => Err(io_e)?,
            },
        }
    }
}

/// 阻塞读取stdin，
/// 要求至少两次主密码,
/// 至少要求密码字符大于等于6个
/// 返回的字符串为明文
fn init_main_pwd_by_stdin() -> anyhow::Result<String> {
    let mut vec = Vec::with_capacity(2);
    let p = loop {
        if vec.is_empty() {
            println!("{}", "Init main password: Enter or press CTRL+C to exit".yellow());
        } else {
            println!("{}", "Init main password: Enter again or press CTRL+C to exit".yellow());
        }
        // 该并不支持中文，密码字符有所限制，应显式提示
        let rl = loop_read_stdin_ascii_passwd(Some(6))?;
        vec.push(rl);
        // 判定是否两个且相等
        if vec.len() >= 2 {
            if vec[vec.len() - 1] == vec[vec.len() - 2] {
                // 两次相等，返回
                break vec.pop().unwrap();
            } else {
                println!("{}", "> Passwords entered twice do not match, please re-enter".red());
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
fn assert_data_file_ready(data_file_path: &Path) -> anyhow::Result<Storage> {
    match DataFileState::look(data_file_path)? {
        DataFileState::NoStorage => Err(anyhow!(
            "Unable to find the data file with: {}\nYou might want to use 'pnt init' to create a data file",
            data_file_path.display().to_string()
        )),
        DataFileState::NoMainPwd => Err(AppError::DataCorrupted)?,
        DataFileState::Ready(conn) => Ok(conn),
    }
}

/// 等待 stdin输入并校验主密码，当失败到一定次数时
/// 释放sqlite 对文件的连接资源并退出进程
/// 该方法要求所有权，因为内部可能执行 drop(conn关闭文件占用）
/// 该方法要么返回，要么因stdin错误返回Err
fn await_verifier_main_pwd(mut context: PntContext) -> anyhow::Result<PntContext> {
    let verifier = context.build_mpv()?;
    // 后续可设定该值为inner配置项，且重试大于一定次数可选操作... 比如删除库文件？
    for n in 0..ALLOC_INVALID_MAIN_PASS_MAX {
        let mp = loop_read_stdin_ascii_passwd(None)?;
        if verifier.verify(&mp)? {
            // 验证通过，返回SecurityContext
            context.security_context = Some(verifier.load_security_context(&mp)?);
            return Ok(context);
        } else {
            // 校验失败，提示
            let tip = format!("{} ({}/{})", "Invalid Password", n + 1, ALLOC_INVALID_MAIN_PASS_MAX);
            println!("{}", tip.on_dark_red().white())
        }
    }
    // 至此，证明for走完仍为校验通过，进程结束
    context.storage.close(); // 释放sqlite 对文件的连接资源
    Err(AppError::InvalidPassword)?
}
