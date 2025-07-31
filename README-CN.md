# pnt

一个简单密码本（密码管理器）TUI命令行程序

> 依赖 [NerdFont] 正确显示部分图标字体

## 编译到可执行文件

* 可执行二进制程序编译到target
    1. `git clone https://github.com/baifangkual/pnt.git`
    2. `cd ./pnt`
    3. `cargo build --release`

或

* cargo install 到本地 ~/.cargo/bin
    1. `git clone https://github.com/baifangkual/pnt.git`
    2. `cd ./pnt`
    3. `cargo install --path .` (强制覆盖使用 `--force` 参数)

## 运行及使用

* 子命令等help信息 `pnt help [COMMAND]`

* 初始化默认数据文件 `pnt init`

* 使用默认数据文件运行 `pnt`

* 修改数据文件配置 `pnt cfg [OPTIONS]` (可通过 `pnt help cfg` 查看可修改的配置)，目前可选的配置有：
    * `--verify-on-launch <BOOLEAN>` 配置是否在启动时就要求验证主密码，默认值 `true`
    * `--auto-relock-idle-sec <SECONDS>` 配置TUI自动切换到锁定状态所等待的空闲时间，默认值为 `0`(关闭)
    * `--auto-close-idle-sec <SECONDS>` 配置TUI程序自动关闭所等待的空闲时间，默认值为 `0`(关闭)

* TUI界面内按键映射可通过按F1查看（显示当前页面可用的按键映射）

## 说明

### 特征

* 程序TUI运行时有两种状态：LOCK 和 UNLOCK（TUI页面左下角有提示当前状态）
    * 只有主密码通过校验，程序才会进入UNLOCK状态，部分操作仅能在UNLOCK情况下进行（比如查看条目）
    * 在LOCK状态下要进行需UNLOCK状态才能执行的操作时，将会弹出要求验证主密码的页面，主密码验证通过才会进入UNLOCK状态以进行操作
    * 在UNLOCK状态时可通过按下 `l` （默认）键重新进入 LOCK 状态
    * 在配置项 `--verify-on-launch` 为 `true` 时，运行程序时将立即要求验证主密码，验证通过则立即进入UNLOCK状态
* 数据文件中不存储主密码或能推导到主密码的值，忘记主密码将无法解密已加密的条目信息

### 实现

主要通过 [ratatui] 构建 TUI界面，通过 [argon2] 进行主密码hash，通过 [aes-gcm] 进行条目的对称加密，
通过 sqlite 进行条目数据的存储。主密码通过加盐hash后存储到数据文件，条目中部分字段加盐后通过主密码作为key对称加密为密文，
要对已存储条目及配置等进行修改需要求验证主密码，只有主密码经校验通过后内存中才会有明文数据。

### 开发迭代方向

* 更小且更快的二进制可执行程序
* 更强的安全性
* 更漂亮的TUI界面
* 更强的易用性

### 安全性

由于被加密的数据文件为sqlite格式静态二进制数据文件，且存在于本地机器，遂对于攻击者已获取到静态数据文件的情况，
没有对抗暴力破解的手段

### 兼容性

* 该项目仅在 AMD64-Windows11 上进行了测试，其他平台理论支持
* 终端支持备用屏幕和原始模式（这是TUI应用运行的基本要求，绝大多数终端都支持）

### 该项目存在原因

我有这个需求，再加之之前写的程序比较臃肿

### 贡献指南

请创建[Issue]以提交 bug问题报告 或 功能请求 或 优化建议

* bug问题报告请包含：
    1. 操作系统和终端环境
    2. 重现步骤
    3. 预期行为与实际行为

* 功能请求请包含：
    1. 需求场景
    2. 建议实现方案

* 优化建议请包含：
    1. 目前实现的问题
    2. 建议优化方案

## License

[MIT]

[MIT]: ./LICENSE

[Issue]: https://github.com/baifangkual/pnt/issues

[ratatui]: https://github.com/ratatui/ratatui

[NerdFont]: https://www.nerdfonts.com/#home

[argon2]: https://en.wikipedia.org/wiki/Argon2

[aes-gcm]: https://docs.rs/aes-gcm/0.10.3/aes_gcm/index.html