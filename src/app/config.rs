use crate::app::consts::{APP_NAME, CONF_FILE_NAME, DATA_FILE_NAME, ENV_CONF_PATH_NAME};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use crate::app::errors::AppError;

/// 运行时使用的实际 cfg
#[derive(Debug, Eq, PartialEq)]
pub struct Cfg {
    /// 存储各密码的sqlite-db路径, 默认在 app data
    pub date: PathBuf,
    /// 盐-该值不同则即使使用相同的主密码也无法解密内容
    salt: Option<String>,
    /// 在启动时要求主密码
    pub need_main_passwd_on_run: bool,
}

impl Cfg {
    pub fn set_salt(&mut self, salt: String) {
        self.salt = Some(salt);
    }
    pub fn salt(&self) -> Option<String> {
        self.salt.clone()
    }
}

/// 载入配置，磁盘或默认配置
pub fn load_cfg() -> Cfg {
    let toml_cf = load_fill_cfg().unwrap_or_else(|e| panic!("load configuration failed: {}", e));
    Cfg::try_from(toml_cf).unwrap() // 上层已经 panic 这里不会 panic，肯定都已填充
}

impl TryFrom<TomlCfg> for Cfg {
    type Error = anyhow::Error;

    fn try_from(value: TomlCfg) -> Result<Self, Self::Error> {
        let c = Cfg {
            date: value.date.ok_or(AppError::CannotOpenData)?,
            salt: None, // 初始化为 none，运行时应在主密码验证后赋值
            need_main_passwd_on_run: value
                .need_main_passwd_on_run.unwrap(), // 因为default不会panic
        };
        Ok(c)
    }
}

/// app 配置文件
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct TomlCfg {
    /// 存储各密码的sqlite-db路径, 默认在 app data，为防止toml的 " 转义，win路径反斜杠路径应使用 ' 符号
    date: Option<PathBuf>,
    /// 在启动时要求主密码
    need_main_passwd_on_run: Option<bool>,
}

impl Default for TomlCfg {
    fn default() -> Self {
        Self {
            date: Some(default_data_path()),
            need_main_passwd_on_run: Some(false),
        }
    }
}

/// 返回 pnt 的数据文件 全路径
/// 返回的路径可能不存在 数据文件
fn default_data_path() -> PathBuf {
    let mut p = dirs::data_dir().unwrap_or_else(|| panic!("No data dir could be found")); // 非 linux win mac 可能None
    p.push(APP_NAME);
    p.push(DATA_FILE_NAME);
    p // os_d/pnt/pnt.data
}

/// 返回默认的 pnt 配置文件 全路径
/// 该全路径将尝试从 env 中读取
/// 若无或配置所在路径不存在东西
/// 则使用 默认 dirs::config_dir + app_name + conf_name 值
/// 返回的路径可能不存在 配置文件
fn default_conf_path() -> PathBuf {
    let env_cp_or = env::var(ENV_CONF_PATH_NAME)
        .into_iter()
        .map(PathBuf::from)
        .find(|p| p.exists()); // 存在才设定为 env 给定的
    if let Some(env_cp) = env_cp_or {
        env_cp
    } else {
        let mut p = dirs::config_dir() // 非 linux win mac 可能None
            .unwrap_or_else(|| panic!("Cannot find config directory"));
        p.push(APP_NAME);
        p.push(CONF_FILE_NAME);
        p // os_c/pnt/pnt.toml
    }
}

/// 载入配置文件，尝试从磁盘载入，若磁盘配置文件不存在，
/// 则使用默认配置文件值
/// ---
/// 返回的配置文件实体中配置：磁盘上配置 优先于 默认配置值,
/// 即磁盘上未有的配置会被 default Some 覆盖
fn load_fill_cfg() -> anyhow::Result<TomlCfg> {
    let disk_cfg = try_load_cfg_from_disk()?;
    if disk_cfg.is_some() {
        // to do 合并配置文件 磁盘None 被 default Some 覆盖
        let mut disk_cfg = disk_cfg.unwrap();
        let lazy_default = std::cell::LazyCell::new(TomlCfg::default);
        // fie need batter？ 下因所有权问题，各赋值仅能 clone，应有其他路径赋值之
        if disk_cfg.date.is_none() {
            disk_cfg.date = lazy_default.date.clone();
        }
        if disk_cfg.need_main_passwd_on_run.is_none() {
            disk_cfg.need_main_passwd_on_run = lazy_default.need_main_passwd_on_run;
        }
        Ok(disk_cfg)
    } else {
        Ok(TomlCfg::default())
    }
}

/// 从磁盘载入配置文件，若配置文件存在则载入，若不存在则 Ok(None)，io错误将 Err
fn try_load_cfg_from_disk() -> anyhow::Result<Option<TomlCfg>> {
    let cp = default_conf_path();
    if cp.exists() {
        let c_str = std::fs::read_to_string(&cp)?;
        Ok(Some(toml::from_str::<TomlCfg>(&c_str)?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let conf = try_load_cfg_from_disk();
        // println!("{:#?}", conf);
        assert!(conf.is_ok());
    }
    
}
