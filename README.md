# pnt

A simple password note (password manager) TUI command-line application

[README-让我们说中文]

> Requires [NerdFont] for proper display of certain icon fonts

## Compiling to Executable

* Compile executable binary to target
    1. `git clone https://github.com/baifangkual/pnt.git`
    2. `cd ./pnt`
    3. `cargo build --release`

or

* Install via cargo to local ~/.cargo/bin
    1. `git clone https://github.com/baifangkual/pnt.git`
    2. `cd ./pnt`
    3. `cargo install --path .` (Use `--force` parameter to force overwrite)

## Running and Usage

* View subcommand help: `pnt help [COMMAND]`

* Initialize default data file: `pnt init`

* Run with default data file: `pnt`

* Modify data file configuration: `pnt cfg [OPTIONS]` (view configurable options via `pnt help cfg`). Current
  configurable options:
    * `--verify-on-launch <BOOLEAN>` Configure whether to require main password verification at launch. Default: `true`
    * `--auto-relock-idle-sec <SECONDS>` Configure idle time before TUI automatically locks. Default: `0` (disabled)
    * `--auto-close-idle-sec <SECONDS>` Configure idle time before TUI automatically closes. Default: `0` (disabled)

* View in-TUI key mappings by pressing F1 (displays available key mappings for current page)

## Notes

### Features

* Two states during TUI operation: LOCK and UNLOCK (indicated at bottom-left of TUI)
    * UNLOCK state requires successful main password verification. Certain operations require UNLOCK state (e.g.,
      viewing entries)
    * Attempting UNLOCK-restricted operations in LOCK state triggers main password verification
    * Press `l` (default) in UNLOCK state to return to LOCK state
    * When `--verify-on-launch` is `true`, main password verification occurs immediately at launch
* Data files store neither the main password nor derivable values. Forgetting the main password will permanently prevent
  decryption of entries

### Implementation

Built with [ratatui] for TUI interface, [argon2] for main password hashing, [aes-gcm] for symmetric entry encryption,
and SQLite for data storage. Main password is salted/hashed before storage. Entry fields are salted/encrypted using main
password as key.
Modifying stored entries requires main password verification - plaintext data exists only in memory after successful
verification.

### Project Iterative direction

* Smaller and faster executable binaries
* Enhanced security
* Improved TUI aesthetics
* Better usability

### Security

Encrypted SQLite data files reside locally as static binary files. If attackers obtain static data files,
there are no anti-bruteforce measures implemented.

### Compatibility

* Tested only on AMD64-Windows11. Theoretically supports other platforms.
* Requires terminal support for alternate screen and raw mode (standard requirements for TUI apps, supported by most
  terminals)

### Why this Project Exists

Personal need for such a tool, plus dissatisfaction with previous bloated implementations

### Contribution Guidelines

Submit bug reports/feature requests/suggestions via [Issue]:

* Bug reports should include:
    1. OS and terminal environment
    2. Reproduction steps
    3. Expected vs actual behavior

* Feature requests should include:
    1. Use case scenario
    2. Proposed implementation

* Optimization suggestions should include:
    1. Current implementation issues
    2. Proposed optimization approach

## License

[MIT]

[MIT]: ./LICENSE

[Issue]: https://github.com/baifangkual/pnt/issues

[ratatui]: https://github.com/ratatui/ratatui

[NerdFont]: https://www.nerdfonts.com/#home

[argon2]: https://en.wikipedia.org/wiki/Argon2

[aes-gcm]: https://docs.rs/aes-gcm/0.10.3/aes_gcm/index.html

[README-让我们说中文]: ./README-CN.md