/// 环境变量中指向的 配置文件路径 key
pub const ENV_CONF_PATH_NAME: &str = "PNT_CONF";
/// app name
pub const APP_NAME: &str = clap::crate_name!();
/// pnt 加密数据文件名
pub const DATA_FILE_NAME: &str = "pntdata";
/// pnt 配置文件名
pub const CONF_FILE_NAME: &str = "pnt.toml";

// ================================================

/// 允许的最多输错主密码次数
pub const ALLOC_VALID_MAIN_PASS_MAX: u8 = 3;

/// 主密码存储名，后续应通过盐hash
pub const MAIN_PASS_KEY: &str = "mp"; // todo hash save
