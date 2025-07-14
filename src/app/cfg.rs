use crate::app::consts::{APP_NAME, CONF_FILE_NAME, DATA_FILE_NAME, ENV_CONF_PATH_KEY, ENV_DEFAULT_DATA_FILE_PATH_KEY};
use crate::app::storage::{kv_cfg::BitCfg, Storage};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Display;
use std::path::{Path, PathBuf};

/// 运行时使用的实际 cfg
#[derive(Debug)]
pub struct Cfg {
    /// pnt data file - 运行时或可被参数替换，否则从配置文件或env找路径
    pub load_data: PathBuf,
    /// 内部配置，从 data file 中读取
    pub inner_cfg: InnerCfg,
}

#[derive(Debug)]
pub struct InnerCfg {
    /// 在运行的时候立即要求主密码
    pub need_main_pwd_on_run: bool,
    pub auto_re_lock_idle_sec: Option<u32>,
    pub auto_close_idle_sec: Option<u32>,
}

impl InnerCfg {
    /// 将配置文件的 inner_cfg 覆盖
    pub fn overwrite_default(&mut self, storage: &Storage) -> anyhow::Result<()> {
        let bf_or = storage.query_cfg_bit_flags()?;
        if let Some(bf) = bf_or {
            self.need_main_pwd_on_run = bf.contains(BitCfg::NEED_MAIN_PWD_ON_RUN);
        }
        self.auto_re_lock_idle_sec = storage.query_cfg_auto_re_lock_idle_sec()?;
        self.auto_close_idle_sec = storage.query_cfg_auto_close_idle_sec()?;
        Ok(())
    }

    /// 将 inner_cfg 存储到db中 (存储 inner cfg，插入或刷新)
    pub fn save_to_data(&self, storage: &mut Storage) {
        // bitflag
        let mut bf = BitCfg::empty();
        // need main on run
        if self.need_main_pwd_on_run {
            bf.insert(BitCfg::NEED_MAIN_PWD_ON_RUN);
        }
        // store
        storage.store_cfg_bit_flags(bf);
        storage.store_cfg_auto_re_lock_idle_sec(self.auto_re_lock_idle_sec.unwrap_or(0));
        storage.store_cfg_auto_close_idle_sec(self.auto_close_idle_sec.unwrap_or(0));
    }
}

impl Display for InnerCfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "need_main_pwd_on_run = {}", self.need_main_pwd_on_run)?;
        writeln!(f, "auto_re_lock_idle_sec = {}", self.auto_re_lock_idle_sec.unwrap_or(0))?;
        writeln!(f, "auto_close_idle_sec = {}", self.auto_close_idle_sec.unwrap_or(0))?;
        Ok(())
    }
}

/// Inner 配置 的 默认配置，data file 中没有的，使用默认配置
impl Default for InnerCfg {
    fn default() -> Self {
        Self {
            need_main_pwd_on_run: true,
            auto_re_lock_idle_sec: None,
            auto_close_idle_sec: None,
        }
    }
}

/// 载入配置，返回的配置不存在的值将会由默认值补充,
/// 若存在一个配置文件，但载入磁盘配置过程发生错误，则返回Err
///
/// ### 载入顺序
///
/// 1. 若 env 给定了指向的配置文件地址则尝试读取其，
/// 若其不存在则使用默认值
/// 2. 若 env 未给定配置文件地址则尝试默认的配置文件地址读取其
/// 若其不存在则使用默认值
/// 3. 若找不到任何可用的配置文件地址，则使用默认值
///
/// 另需注意，即使找到了配置文件，若其中未配置default_data，
/// 则尝试寻找 env 中指定的 数据文件地址，
/// 若 env 中 未指定数据文件地址，则使用默认数据文件地址,
///
/// **即两个关键位置 寻找顺序：**
///
/// cfg: env -> default
/// data: env.cfg.default_data -> default.cfg.default_data -> env
///
/// 该方法返回Result是因为可能给定了一个地址但该地址文件无权读取或其无法解析为toml文件，
/// 这种情况发生时，返回Err
///
pub fn load_cfg() -> anyhow::Result<Cfg> {
    env_conf_path()
        .or_else(default_conf_path)
        .map(|path| load_cfg_with_path(&path))
        // 下 or_else: 完全无法找到任何可读conf：使用完全的 default配置
        .unwrap_or_else(|| Ok(Cfg::from(TomlCfg::default())))
}

/// 从指定位置载入配置，返回的配置不存在的值将会由默认值补充
///
/// 无权读取或其无法解析为toml文件返回Err
pub fn load_cfg_with_path(path: &Path) -> anyhow::Result<Cfg> {
    Ok(Cfg::from(try_load_cfg_from_disk(path)?.unwrap_or_default()))
}

impl From<TomlCfg> for Cfg {
    /// Cfg 由该过程从TomlCfg转换，
    /// toml文件中未配置的缺省，由默认值填充
    /// inner_cfg 完全不由 toml 配置，而是后续从 data file 中 覆盖默认值
    fn from(value: TomlCfg) -> Self {
        Cfg {
            load_data: value
                .default_data
                .or_else(env_data_path) // toml 中未配置，尝试使用 env_data_path
                .unwrap_or_else(default_data_path), // toml 未配置，env未有，尝试使用默认路径位置，即toml中配置优先级最高
            inner_cfg: InnerCfg::default(),
        }
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
        Self { default_data: None }
    }
}

/// 从磁盘载入配置文件，若配置文件存在则载入，
/// 若不存在则 Ok(None)，io错误将 Err（包括无权读取文件及无法解析为toml）
pub fn try_load_cfg_from_disk(cp: &Path) -> anyhow::Result<Option<TomlCfg>> {
    if cp.exists() {
        let c_str = std::fs::read_to_string(cp)?;
        Ok(Some(toml::from_str::<TomlCfg>(&c_str)?))
    } else {
        Ok(None)
    }
}

/// 从env中寻找配置要求的 pnt conf
/// 返回的Path即使指向的位置没有或不是一个有效文件，也返回Some，
/// None仅代表没有该环境变量项
pub fn env_conf_path() -> Option<PathBuf> {
    env::var(ENV_CONF_PATH_KEY).into_iter().map(PathBuf::from).next_back() // 不检查 PathBuf指向的位置是否有效，只要环境变量配置存在，则覆盖默认配置文件中找的行为
}

/// 返回默认配置文件路径，None表示无法生成默认配置文件路径
pub fn default_conf_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut p| {
        p.push(APP_NAME);
        p.push(CONF_FILE_NAME);
        p
    })
}

/// 从env中寻找配置要求的default_data file path
/// 返回的Path即使指向的位置没有或不是一个有效data file，也返回Some，
/// None仅代表没有该环境变量项
pub fn env_data_path() -> Option<PathBuf> {
    env::var(ENV_DEFAULT_DATA_FILE_PATH_KEY)
        .into_iter()
        .map(PathBuf::from)
        .next_back() // 不检查 PathBuf指向的位置是否有效，只要环境变量配置存在，则覆盖默认配置文件中找的行为
}

/// 返回 默认 数据文件 全路径
/// 返回的路径可能不存在 数据文件
/// 该方法不尝试从env中找，仅找寻dirs数据目录位置
///
/// # Panics
///
/// 完全无法生成可用的数据文件路径时
/// 非 linux win mac 可能None
/// 数据文件找不到应直接终止进程，因为完全依赖数据文件运行
pub fn default_data_path() -> PathBuf {
    dirs::data_dir()
        .map(|mut p| {
            p.push(APP_NAME);
            p.push(DATA_FILE_NAME);
            p
        })
        .unwrap_or_else(|| panic!("Cannot found data file"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let conf = load_cfg();
        // println!("{:#?}", conf);
        assert!(conf.is_ok());
    }
}
