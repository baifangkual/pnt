use crate::app::consts::{APP_NAME, CONF_FILE_NAME, DATA_FILE_NAME, ENV_CONF_PATH_KEY, ENV_DEFAULT_DATA_FILE_PATH_KEY};
use crate::app::errors::AppError;
use crate::app::storage::{Storage, kv_cfg::BitCfg};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

/// 运行时使用的实际 cfg
#[derive(Debug)]
pub struct Cfg {
    /// 存储各密码的sqlite-db路径, 默认在 app data
    pub default_date: PathBuf,
    /// 内部配置，从 data file 中读取
    pub inner_cfg: InnerCfg,
}

impl Cfg {
    /// 将配置文件的 inner_cfg 覆盖
    pub fn overwrite_inner_cfg(&mut self, storage: &Storage) -> anyhow::Result<()> {
        let bcf = storage.query_cfg_bit_flags()?;
        if bcf.is_empty() {
            Ok(())
        } else {
            if bcf.contains(BitCfg::NEED_MAIN_ON_RUN) {
                self.inner_cfg.need_main_passwd_on_run = true;
            }
            Ok(())
        }
    }

    /// 将 inner_cfg 存储到db中
    pub fn store_inner_cfg(&self, storage: &mut Storage) -> anyhow::Result<()> {
        todo!("impl storage inner_cfg")
    }
}

#[derive(Debug)]
pub struct InnerCfg {
    /// 在运行的时候立即要求主密码
    pub need_main_passwd_on_run: bool,
}

/// Inner 配置 的 默认配置，data file 中没有的，使用默认配置
impl Default for InnerCfg {
    fn default() -> Self {
        Self {
            need_main_passwd_on_run: true,
        }
    }
}

/// 载入配置，返回的配置不存在的值将会由默认值补充
pub fn load_cfg() -> anyhow::Result<Cfg> {
    load_cfg_with_path(&default_conf_path())
}

/// 从指定位置载入配置，返回的配置不存在的值将会由默认值补充
pub fn load_cfg_with_path(path: &Path) -> anyhow::Result<Cfg> {
    let toml_cf = load_fill_cfg(path)?;
    Cfg::try_from(toml_cf)
}

impl TryFrom<TomlCfg> for Cfg {
    type Error = anyhow::Error;

    fn try_from(value: TomlCfg) -> Result<Self, Self::Error> {
        let exists_p;
        if let Some(p) = value.default_data {
            exists_p = p; // 配置不会判定路径指向文件是否存在及是否可用
        } else {
            // 不存在，Err
            return Err(AppError::CannotGetDataFilePath)?;
        };
        let c = Cfg {
            default_date: exists_p,
            inner_cfg: InnerCfg::default(),
        };
        Ok(c)
    }
}

/// app 配置文件
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub(super) struct TomlCfg {
    /// 存储各密码的sqlite-db路径, 默认在 app data，为防止toml的 " 转义，win路径反斜杠路径应使用 ' 符号
    pub(super) default_data: Option<PathBuf>,
}

impl Default for TomlCfg {
    fn default() -> Self {
        Self {
            default_data: Some(default_data_path()),
        }
    }
}

/// 返回 pnt 的数据文件 全路径
/// 返回的路径可能不存在 数据文件
/// 该方法不尝试从env中找，仅找寻dirs数据目录位置
pub fn default_data_path() -> PathBuf {
    let mut p = dirs::data_dir().unwrap_or_else(|| panic!("No data dir could be found")); // 非 linux win mac 可能None
    p.push(APP_NAME);
    p.push(DATA_FILE_NAME);
    p // os_d/pnt/pnt.data
}

/// 返回默认的 pnt 配置文件 全路径
/// 该全路径将尝试从 env 中读取
/// 若无 则使用 默认 dirs::config_dir + app_name + conf_name 值
/// 返回的路径可能不存在 配置文件
pub fn default_conf_path() -> PathBuf {
    let env_cp_or = env_conf_path();
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

/// 从磁盘载入配置文件，若配置文件存在则载入，若不存在则 Ok(None)，io错误将 Err
pub fn try_load_cfg_from_disk(cp: &Path) -> anyhow::Result<Option<TomlCfg>> {
    if cp.exists() {
        let c_str = std::fs::read_to_string(&cp)?;
        Ok(Some(toml::from_str::<TomlCfg>(&c_str)?))
    } else {
        Ok(None)
    }
}

/// 从env中寻找配置要求的 pnt conf
/// 返回的Path即使指向的位置没有或不是一个有效文件，也返回Some，
/// None仅代表没有该环境变量项
pub fn env_conf_path() -> Option<PathBuf> {
    env::var(ENV_CONF_PATH_KEY).into_iter().map(PathBuf::from).last() // 不检查 PathBuf指向的位置是否有效，只要环境变量配置存在，则覆盖默认配置文件中找的行为
}

/// 从env中寻找配置要求的default_data file path
/// 返回的Path即使指向的位置没有或不是一个有效data file，也返回Some，
/// None仅代表没有该环境变量项
pub fn env_data_file_path() -> Option<PathBuf> {
    env::var(ENV_DEFAULT_DATA_FILE_PATH_KEY)
        .into_iter()
        .map(PathBuf::from)
        .last() // 不检查 PathBuf指向的位置是否有效，只要环境变量配置存在，则覆盖默认配置文件中找的行为
}

/// 载入配置文件，尝试从磁盘载入，若磁盘配置文件不存在，
/// 则使用默认配置文件值
/// ---
/// 返回的配置文件实体中配置：磁盘上配置 优先于 默认配置值,
/// 即磁盘上未有的配置会被 default Some 覆盖
fn load_fill_cfg(cfg_path: &Path) -> anyhow::Result<TomlCfg> {
    let disk_cfg = try_load_cfg_from_disk(cfg_path)?;
    let env_data_file_path_or = env_data_file_path();
    if let Some(mut disk_cfg) = disk_cfg {
        // env data file path 优先级最高
        if let Some(env_dfp) = env_data_file_path_or {
            disk_cfg.default_data = Some(env_dfp);
        }
        // to do 合并配置文件 磁盘None 被 default Some 覆盖
        let lazy_default = std::cell::LazyCell::new(TomlCfg::default);
        // fie need batter？ 下因所有权问题，各赋值仅能 clone，应有其他路径赋值之
        if disk_cfg.default_data.is_none() {
            disk_cfg.default_data = lazy_default.default_data.clone();
        }
        Ok(disk_cfg)
    } else {
        let mut default_cfg = TomlCfg::default();
        // env data file path 优先级最高
        if let Some(env_dfp) = env_data_file_path_or {
            default_cfg.default_data = Some(env_dfp);
        }
        Ok(default_cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let conf = try_load_cfg_from_disk(&default_conf_path());
        // println!("{:#?}", conf);
        assert!(conf.is_ok());
    }
}
