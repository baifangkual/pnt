use crate::app::cfg::load_cfg;
use crate::app::consts::{ALLOC_INVALID_MAIN_PASS_MAX, APP_NAME};
use crate::app::context::{DataFileState, PntContext};
use crate::app::crypto::{Encrypter, MainPwdEncrypter, MainPwdVerifier, build_mpv};
use crate::app::errors::AppError;
use crate::app::storage::Storage;
use anyhow::anyhow;
use clap::Args;
use clap::{Parser, Subcommand};
use ratatui::crossterm::style::Stylize;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// runtime cli args...
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// 要求使用的数据文件
    #[arg( global = true ,short='d',long= "data", value_name = "DATA_FILE", help = Self::CLI_HELP_DATA
    )]
    data: Option<PathBuf>,
    /// 要通过 about 值模糊查找的 条目
    #[arg(short = 'f', long = "find", value_name = "ABOUT", help = Self::CLI_HELP_FIND)]
    find: Option<String>,
    /// 子命令
    #[command(subcommand)]
    sub_command: Option<SubCmd>,
}

impl CliArgs {
    const CLI_HELP_DATA: &'static str = "Use the specified data file,
if this parameter is not provided,
Use the default data file (default_data)";
    const CLI_HELP_FIND: &'static str = "Find for entries with similar 'about' values";
}

/// 子命令定义
#[derive(Subcommand, Debug)]
enum SubCmd {
    /// Print the default data file location (default_data)
    #[command(name = "default")]
    Default,
    /// 子命令 初始化一个 data file
    #[command(name = "init",
    about = Self::SUB_INIT_HELP_HEAD,
    long_about = Self::SUB_INIT_HELP)]
    Init,
    /// Modify the main password in an interactive context
    #[command(name = "mmp", long_flag = "modify-main-pwd")]
    ModifyMainPwd,
    /// 子命令 print 或 修改 cfg
    #[command(name = "cfg",
    long_flag = "data-file-cfg",
    about = Self::SUB_CFG_HELP_HEAD,
    long_about = Self::SUB_CFG_HELP)]
    Cfg(SubCmdCfgArgs),
}

impl SubCmd {
    const SUB_INIT_HELP_HEAD: &'static str = "Initializing data file storage location";
    const SUB_INIT_HELP: &'static str = "Initializing data file storage location.
\nDefault Data file initialization location search sequence:
.1. The `default_data` value in the configuration file (ENV`PNT_CONF_FILE`)
.2. The `default_data` value in the configuration file (default config file)
.3. The value specified by the environment variable `PNT_DEFAULT_DATA_FILE`
.4. Default path";
    const SUB_CFG_HELP_HEAD: &'static str = "Management of configuration related to specific data files";
    const SUB_CFG_HELP: &'static str = "Management of configuration related to specific data files.
\nIf no configuration parameters are specified for setting,
it will print the current state of all configurations.";
}

#[derive(Args, Debug)]
struct SubCmdCfgArgs {
    /// Setting whether to require the main password immediately at runtime
    #[arg(long = "modify--need-main-pwd-on-run", value_name = "BOOLEAN")]
    modify_need_main_pwd_on_run: Option<bool>,
    /// Setting how many seconds of inactivity before the TUI re-enters the Lock state (set to 0 to disable)
    #[arg(long = "modify--auto-re-lock-idle-sec", value_name = "SECONDS")]
    modify_auto_re_lock_idle_sec: Option<u32>,
    /// Setting how many seconds of inactivity before the TUI automatically closes (set to 0 to disable)
    #[arg(long = "modify--auto-close-idle-sec", value_name = "SECONDS")]
    modify_auto_close_idle_sec: Option<u32>,
}

impl CliArgs {
    /// cli 运行，若Err，则应向main返回要求进程非成功退出，
    /// 若Ok(Some(context))则表明要求TUI运行，
    /// 若OK(None) 则表明成功cli运行结束，程序成功退出
    pub fn run(&self) -> anyhow::Result<Option<PntContext>> {
        // sub-cmd: default
        if let Some(SubCmd::Default) = &self.sub_command {
            println!("Default Data file: {}", load_cfg()?.load_data.display());
            return Ok(None);
        }

        // 看看参数要求
        if let Some(SubCmd::Init) = &self.sub_command {
            // 显式要求 init
            handle_pnt_data_init(self.data.clone())?;
            return Ok(None);
        }

        // =======================================
        // CONTEXT BUILD =========================
        // =======================================
        let mut cfg = load_cfg()?;

        // 若有 cli 参数 --data 则替换cfg中的
        if let Some(data) = &self.data {
            cfg.load_data = data.clone()
        };
        // 连接数据文件，因为为非显式init，所以任何失败情况该方法内均Err向上回报
        let conn = assert_data_file_ready(&cfg.load_data)?;
        // 已填充inner配置的cfg
        cfg.inner_cfg.overwrite_default(&conn)?;

        // pnt 上下文
        let mut context = PntContext::new_with_un_verified(cfg, conn);
        // =======================================
        // CONTEXT BUILD =========================
        // =======================================

        if let Some(SubCmd::ModifyMainPwd) = &self.sub_command {
            // 要求修改主密码...
            println!("Data file: '{}'", context.storage.path().unwrap());
            println!(
                "{}",
                "Verify the current data file main password to modify main password".yellow()
            );
            // 因为要修改主密码，遂立即要求主密码
            let context = await_verifier_main_pwd(context)?;

            // 至此 原主密码已校验
            let new_mp = setting_main_pwd_by_stdin("New main password")?;

            // 不可反驳解构 PNT CONTEXT，因为已经校验了主密码，所以 else 一定不会发生
            let PntContext {
                storage,
                security_context: Some(old_sec_ctx),
                ..
            } = context
            else {
                unreachable!("因上述await_verifier_main_pwd，不会执行到该分支")
            };

            let new_b64_s_mph = MainPwdEncrypter::new_from_random_salt().encrypt(new_mp.clone())?;
            println!("\nNew main password hash:\n{new_b64_s_mph}\n");
            let new_sec_ctx = MainPwdVerifier::from_b64_s_mph(&new_b64_s_mph)?.load_security_context(&new_mp)?;

            // 当前线程卡在这，等待数据库文件内容更新返回 =====
            println!("{}", "...modify main password...\n".grey());
            storage.update_b64_s_mph(new_b64_s_mph, old_sec_ctx, new_sec_ctx)?;
            println!("{}", "Successfully modify main password".green());
            // 当前线程卡在这，等待数据库文件内容更新返回 =====

            return Ok(None);
        } else if let Some(SubCmd::Cfg(args)) = &self.sub_command {
            // 要求修改 inner 配置
            // dbg!(&args);
            // 控制是否 list显示配置（没有任何修改需求时）
            let mut no_any_args = true;

            println!("Data file: '{}'", context.storage.path().unwrap());
            println!(
                "{}",
                "Verify the current data file main password to modify or print its configuration".yellow()
            );
            // 因为要修改配置，遂立即要求主密码
            let mut context = await_verifier_main_pwd(context)?;

            // ===========================================================
            // change inner cfg and store ================================
            // ===========================================================
            if let Some(rs_need_mp_on_run) = &args.modify_need_main_pwd_on_run {
                no_any_args = false;
                context.cfg.inner_cfg.need_main_pwd_on_run = *rs_need_mp_on_run;
                context.cfg.inner_cfg.save_to_data(&mut context.storage);
                println!(
                    "{}",
                    "Successfully modified configuration 'need_main_pwd_on_run'".green()
                );
            }
            if let Some(rs_auto_re_lock_idle_sec) = &args.modify_auto_re_lock_idle_sec {
                no_any_args = false;
                context.cfg.inner_cfg.auto_re_lock_idle_sec = Some(*rs_auto_re_lock_idle_sec);
                context.cfg.inner_cfg.save_to_data(&mut context.storage);
                println!(
                    "{}",
                    "Successfully modified configuration 'auto_re_lock_idle_sec'".green()
                );
            }
            if let Some(rs_auto_close_idle_sec) = &args.modify_auto_close_idle_sec {
                no_any_args = false;
                context.cfg.inner_cfg.auto_close_idle_sec = Some(*rs_auto_close_idle_sec);
                context.cfg.inner_cfg.save_to_data(&mut context.storage);
                println!(
                    "{}",
                    "Successfully modified configuration 'auto_close_idle_sec'".green()
                );
            }
            // ===========================================================
            // change inner cfg and store ================================
            // ===========================================================

            // 修改 cfg时务必修改 该值为 false，当该值为true，打印配置
            if no_any_args {
                println!("cfg:\n{}", context.cfg.inner_cfg)
            }
            // 使用 OK（NONE）打断不使TUI运行
            return Ok(None);
        }

        // cli 要求 find
        if let Some(find) = &self.find {
            if context.is_need_mp_on_run() {
                context = await_verifier_main_pwd(context)?;
            }
            context
                .storage
                .select_entry_by_about_like(find)
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

/// 初始化 pnt 数据文件
///
/// 该方法内将根据cli参数及env及配置文件等设置情况
///
/// ### 优先级
///
/// 明确Cli --data 参数 or -> conf.default_data or -> env -> default
fn handle_pnt_data_init(init_arg_target: Option<PathBuf>) -> anyhow::Result<()> {
    println!("{}", "Data file initialized\n".bold().dark_cyan());
    /*
    // 先从参数 --data 找需要，
    // 若无，则从可能存在的配置文件中找
    // 若无，则顺次从环境变量找，
    // 若无，则使用默认值
     */

    let (data_target_path, target_on_cli_arg) = if let Some(arg_data_path) = init_arg_target {
        (arg_data_path, true)
    } else {
        println!("Initialized default data file (default_data)");
        (load_cfg()?.load_data, false)
    };

    // dbg!(&data_target_path);

    let msg = format!("\nwill create data file with: '{}'", data_target_path.display());
    println!("{}", msg.bold().cyan());
    println!("\npress Enter to init main password with interactive context or press Ctrl-C to exit");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    // 初始化主密码
    let mph = MainPwdEncrypter::new_from_random_salt().encrypt(setting_main_pwd_by_stdin("Init main password")?)?;
    println!("{}", "successfully init main password".green());

    // 检查 data local path 位置是否存在文件，若存在，则提示其是否覆盖
    if data_target_path.exists() {
        println!("\nfile '{}'already exists", data_target_path.display());
        println!("{}", "overwrite this file?".red());
        println!("\nenter 'yes' to overwrite existing file or press Ctrl-C to exit");
        buf.clear();
        std::io::stdin().read_line(&mut buf)?;
        if buf.to_lowercase().trim() == "yes" {
            std::fs::remove_file(&data_target_path)?;
        } else {
            return Err(anyhow!(
                "file '{}' already exists,\ncannot create data file",
                data_target_path.display().to_string()
            ));
        }
    }

    println!("\nmain password hash:\n{mph}\n");
    let conn = Storage::open_in_memory()?;
    conn.store_b64_s_mph(&mph);
    // 存储数据文件至指定位置, 该方法不会覆盖文件，位置已有会Err
    conn.db_mem_to_disk(&data_target_path)?;
    let msg = format!("data file created: {}", data_target_path.display());
    println!("{}", msg.bold().cyan());

    if target_on_cli_arg {
        println!("`{} --data {}` to use data file.", APP_NAME, data_target_path.display())
    } else {
        println!("`{}` to use default data file.", APP_NAME)
    }

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
fn setting_main_pwd_by_stdin(prefix: &str) -> anyhow::Result<String> {
    let mut vec = Vec::with_capacity(2);
    let p = loop {
        if vec.is_empty() {
            let prefix_msg = format!("{}: Enter or press CTRL+C to exit", prefix);
            println!("{}", prefix_msg.yellow());
        } else {
            let prefix_msg = format!("{}: Enter again or press CTRL+C to exit", prefix);
            println!("{}", prefix_msg.yellow());
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

/// 等待 stdin输入并校验主密码，该方法要求所有权，因为内部可能执行 drop(conn关闭文件占用），
/// 当失败到一定次数时 释放 storage 对文件的连接资源并退出进程，
/// 该方法要么返回，要么因stdin错误返回Err
fn await_verifier_main_pwd(mut context: PntContext) -> anyhow::Result<PntContext> {
    let verifier = build_mpv(&context.storage)?;
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
