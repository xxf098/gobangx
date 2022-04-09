<div align="center">

gobangx is based on [gobang](https://github.com/TaKO8Ki/gobang)

A cross-platform Vim-like database management tool written in Rust

[![github workflow status](https://img.shields.io/github/workflow/status/xxf098/gobangx/CI/main)](https://github.com/xxf098/gobangx/actions)


</div>

## Features

- Cross-platform support (macOS, Windows, Linux)
- Multiple Database support (MySQL, PostgreSQL, SQLite)
- Intuitive keyboard only control

## Installation

### From binaries (Linux, macOS, Windows)

- Download the [latest release binary](https://github.com/xxf098/gobangx/releases) for your system
- Set the `PATH` environment variable

## Usage

```
$ gobang
```

```
$ gobang -h
USAGE:
    gobang [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config-path <config-path>    Set the config file
```

If you want to add connections, you need to edit your config file. For more information, please see [Configuration](#Configuration).

## Keymap

| Key | Description |
| ---- | ---- |
| <kbd>h</kbd>, <kbd>j</kbd>, <kbd>k</kbd>, <kbd>l</kbd> | Scroll left/down/up/right |
| <kbd>Ctrl</kbd> + <kbd>u</kbd>, <kbd>Ctrl</kbd> + <kbd>d</kbd> | Scroll up/down multiple lines |
| <kbd>0</kbd>, <kbd>$</kbd> | Scroll to start/end |
| <kbd>g</kbd> , <kbd>G</kbd> | Scroll to top/bottom |
| <kbd>f</kbd>+<kbd>_</kbd> | Forward to next column starts with the character `_` |
| <kbd>F</kbd>+<kbd>_</kbd> | Backward to next column starts with the character `_` |
| <kbd>;</kbd> | Repeat previous f, F movement |
| <kbd>H</kbd>, <kbd>J</kbd>, <kbd>K</kbd>, <kbd>L</kbd> | Extend selection by one cell left/down/up/right |
| <kbd>y</kbd> | Yank a cell value |
| <kbd>yc</kbd> | Yank column name |
| <kbd>Y</kbd> | Yank `CREATE TABLE` or `INSERT INTO` sql |
| <kbd>D</kbd> | Delete row by primary key or `id` or first column |
| <kbd>C</kbd> | Change current cell value, set value to `NULL` with `<NULL>` |
| <kbd>:</kbd> | Start ex command, see below for commands list |
| <kbd>o</kbd>, <kbd>O</kbd> | Order column asc/desc |
| <kbd>=</kbd>, <kbd>-</kbd> | Expand/Shorten column width |
| <kbd>←</kbd>, <kbd>→</kbd> | Move focus to left/right |
| <kbd>c</kbd> | Move focus to connections |
| <kbd>/</kbd> | Filter |
| <kbd>?</kbd> | Help |
| <kbd>1</kbd>, <kbd>2</kbd>, <kbd>3</kbd>, <kbd>4</kbd>, <kbd>5</kbd> | Switch to records/columns/constraints/foreign keys/indexes tab |

## Command
| Command | Description |
| ---- | ---- |
| <kbd>tree</kbd> | Toggle database tree |

## Configuration

The location of the file depends on your OS:

- macOS: `$HOME/.config/gobang/config.toml`
- Linux: `$HOME/.config/gobang/config.toml`
- Windows: `%APPDATA%/gobang/config.toml`

The following is a sample config.toml file:

```toml
[[conn]]
type = "mysql"
user = "root"
host = "localhost"
port = 3306

[[conn]]
type = "mysql"
user = "root"
host = "localhost"
port = 3306
password = "password"
database = "foo"

[[conn]]
type = "postgres"
user = "root"
host = "localhost"
port = 5432
database = "bar"

[[conn]]
type = "sqlite"
path = "/path/to/baz.db"

[settings]
# support: red,green,yellow,blue,magenta,cyan or color code 
color = "red"
# page size limit, page_size >= 20 && page_size <= 2000
page_size = 100
```
