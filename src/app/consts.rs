use indoc::indoc;

/// 环境变量中指向的 配置文件路径 key （值优先级高于默认配置文件位置）
pub const ENV_CONF_PATH_KEY: &str = "PNT_CONF_FILE";
/// 环境变量中指向的 默认 data file 位置 key（值优先级高于配置文件中的）
pub const ENV_DEFAULT_DATA_FILE_PATH_KEY: &str = "PNT_DEFAULT_DATA_FILE";
/// app name
pub const APP_NAME: &str = clap::crate_name!();
pub const APP_NAME_AND_VERSION: &str = concat!(clap::crate_name!(), " v", clap::crate_version!());
/// pnt 加密数据文件名
pub const DATA_FILE_NAME: &str = "pntdata";
/// pnt 配置文件名
pub const CONF_FILE_NAME: &str = "pnt.toml";

// ================================================

/// 允许的最多输错主密码次数
pub const ALLOC_INVALID_MAIN_PASS_MAX: u8 = 3;

/// 锁图标
#[allow(unused)]
pub const LOCK_ICON: &str = indoc! {"
   ▄▄▄
  █   █
█▀▀▀▀▀▀▀█
█   ▄   █
█   ▀   █
▀▀▀▀▀▀▀▀▀
"};

/// 钥匙图标
pub const KEY_ICON: &str = indoc! {"
 ▄▄
█  █▀▀▀▀▀▀█▀
 ▀▀
"};

pub const KEY_LEFT_ICON: &str = indoc! {"
█▀▀▀▀▀▀█
█  ▀▀  █
█▄▄▄▄▄▄█
  █  █
  █ ▀▄
  █ ▀▄
   ▀▀▀
"};
