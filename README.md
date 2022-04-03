<div align="center">

gobangx is based on [gobang](https://github.com/TaKO8Ki/gobang)

A cross-platform TUI database management tool written in Rust with more useful features

[![github workflow status](https://img.shields.io/github/workflow/status/xxf098/gobangx/CI/main)](https://github.com/xxf098/gobangx/actions)


</div>

## Features

- Cross-platform support (macOS, Windows, Linux)
- Multiple Database support (MySQL, PostgreSQL, SQLite)
- Intuitive keyboard only control

## What does "gobang" come from?

gobang means a Japanese game played on goban, a go board. The appearance of goban looks like table structure. And I live in Kyoto, Japan. In Kyoto city, streets are laid out on a grid (We call it “goban no me no youna (碁盤の目のような)”). They are why I named this project "gobang".

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
| <kbd>g</kbd> , <kbd>G</kbd> | Scroll to top/bottom |
| <kbd>H</kbd>, <kbd>J</kbd>, <kbd>K</kbd>, <kbd>L</kbd> | Extend selection by one cell left/down/up/right |
| <kbd>y</kbd> | Copy a cell value |
| <kbd>Y</kbd> | Copy `CREATE TABLE` or `INSERT INTO` sql |
| <kbd>D</kbd> | Delete row by primary key or `id` or first column |
| <kbd>C</kbd> | Change current cell value, set value to `NULL` with `<NULL>` |
| <kbd>o</kbd>, <kbd>O</kbd> | Order column asc/desc |
| <kbd>=</kbd>, <kbd>-</kbd>, <kbd>0</kbd> | expand, shorten, reset column width |
| <kbd>←</kbd>, <kbd>→</kbd> | Move focus to left/right |
| <kbd>c</kbd> | Move focus to connections |
| <kbd>/</kbd> | Filter |
| <kbd>?</kbd> | Help |
| <kbd>1</kbd>, <kbd>2</kbd>, <kbd>3</kbd>, <kbd>4</kbd>, <kbd>5</kbd> | Switch to records/columns/constraints/foreign keys/indexes tab |

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

[theme]
# support: red,green,yellow,blue,magenta,cyan or color code 
color = "red"
```

## Contribution

Contributions, issues and pull requests are welcome!
