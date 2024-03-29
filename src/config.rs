use crate::log::LogLevel;
use crate::event::Key;
use crate::cli::CliConfig;
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use url::Url;
use tui::style::Color;

#[cfg(test)]
use serde::Serialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub conn: Vec<Connection>,
    #[serde(default)]
    pub key_config: KeyConfig,
    #[serde(default)]
    pub log_level: LogLevel,
    #[serde(default)]
    pub settings: Settings,
}

// TODO: Oracle, SQL Server
#[derive(Debug, Deserialize, Clone)]
pub enum DatabaseType {
    #[serde(rename = "mysql")]
    MySql,
    #[serde(rename = "postgres")]
    Postgres,
    #[serde(rename = "sqlite")]
    Sqlite,
    #[serde(rename = "mssql")]
    Mssql,  // TODO: remove Mssql
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MySql => write!(f, "mysql"),
            Self::Postgres => write!(f, "postgres"),
            Self::Sqlite => write!(f, "sqlite"),
            Self::Mssql => write!(f, "mssql"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            conn: vec![Connection {
                r#type: DatabaseType::MySql,
                name: None,
                user: Some("root".to_string()),
                host: Some("localhost".to_string()),
                port: Some(3306),
                path: None,
                password: None,
                database: None,
                ssl_mode: None,
                database_url: None,
            }],
            key_config: KeyConfig::default(),
            log_level: LogLevel::default(),
            settings: Settings::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Connection {
    r#type: DatabaseType,
    name: Option<String>,
    user: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    path: Option<std::path::PathBuf>, // sqlite file path
    password: Option<String>,
    ssl_mode: Option<String>, // mysql ssl-mode, postgress sslmode
    pub database: Option<String>,
    pub database_url: Option<String>, // database connection url
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(Serialize))]
pub struct KeyConfig {
    pub scroll_up: Key,
    pub scroll_down: Key,
    pub scroll_right: Key,
    pub scroll_left: Key,
    pub move_up: Key,
    pub move_down: Key,
    pub copy: Key,
    pub copy2: [Key; 2], // copy column
    pub advanced_copy: Key, // copy table schema
    pub enter: Key,
    pub space: Key,
    pub exit: Key,
    pub quit: Key,
    pub exit_popup: Key,
    pub focus_right: Key,
    pub focus_left: Key,
    pub focus_above: Key,
    pub focus_connections: Key,
    pub focus_recents: Key,
    pub open_help: Key,
    pub filter: Key,
    pub scroll_down_multiple_lines: Key,
    pub scroll_up_multiple_lines: Key,
    pub scroll_to_top: Key,
    pub scroll_to_bottom: Key,
    pub extend_selection_by_one_cell_left: Key,
    pub extend_selection_by_one_cell_right: Key,
    pub extend_selection_by_one_cell_up: Key,  // Jump to the first child of the current nodes parent.
    pub extend_selection_by_one_cell_down: Key, // Jump to the last child of the current nodes parent.
    pub tab_records: Key,
    pub tab_columns: Key,
    pub tab_constraints: Key,
    pub tab_foreign_keys: Key,
    pub tab_indexes: Key,
    pub tab_sql_editor: Key,
    pub tab_properties: Key,
    pub extend_or_shorten_widget_width_to_right: Key,
    pub extend_or_shorten_widget_width_to_left: Key,
    pub expand_column_width: Key,
    pub shorten_column_width: Key,
    // pub reset_column_width: [Key; 2],
    pub delete: Key, // drop table/ delete record by primary key
    pub edit_cell: Key, // edit table cell
    pub edit_command: Key, // toggle command
    pub orderby_asc: Key,
    pub orderby_desc: Key,
    pub jump_to_start: Key,
    pub jump_to_end: Key,
    pub forward: Key,
    pub backward: Key,
    pub repeat_movement: Key,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    #[serde(deserialize_with = "deserialize_color", default="default_color")]
    pub color: Color,
    #[serde(deserialize_with = "deserialize_page_size", default="default_page_size")]
    pub page_size: u16,
}

// https://brokenco.de/2020/08/03/serde-deserialize-with-string.html
fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error> where D: Deserializer<'de> {
    let buf = String::deserialize(deserializer)?;
    let color = match buf.as_ref() {
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        x if x.parse::<u8>().is_ok() => Color::Indexed(x.parse::<u8>().unwrap()),
        _ => Color::Blue,
    };
   Ok(color)
}

fn default_color() -> Color {
    Color::Blue
}

fn deserialize_page_size<'de, D>(deserializer: D) -> Result<u16, D::Error> where D: Deserializer<'de> {
    let mut page_size = u16::deserialize(deserializer)?;
    page_size = page_size.max(20);
    page_size = page_size.min(2000);
    Ok(page_size)
}

fn default_page_size() -> u16 {
    200
}

impl Default for Settings {

    fn default() -> Self {
        Self { color: Color::Blue, page_size: 200 }
    }
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            scroll_up: Key::Char('k'),
            scroll_down: Key::Char('j'),
            scroll_right: Key::Char('l'),
            scroll_left: Key::Char('h'),
            move_up: Key::Up,
            move_down: Key::Down,
            copy: Key::Char('y'),
            copy2: [Key::Char('y'), Key::Char('c')],
            advanced_copy: Key::Char('Y'),
            enter: Key::Enter,
            space: Key::Char(' '),
            exit: Key::Ctrl('c'),
            quit: Key::Char('q'),
            exit_popup: Key::Esc,
            focus_right: Key::Right,
            focus_left: Key::Left,
            focus_above: Key::Up,
            focus_connections: Key::Char('c'),
            focus_recents: Key::Char('r'),
            open_help: Key::Char('?'),
            filter: Key::Char('/'),
            scroll_down_multiple_lines: Key::Ctrl('d'),
            scroll_up_multiple_lines: Key::Ctrl('u'),
            scroll_to_top: Key::Char('g'),
            scroll_to_bottom: Key::Char('G'),
            extend_selection_by_one_cell_left: Key::Char('H'),
            extend_selection_by_one_cell_right: Key::Char('L'),
            extend_selection_by_one_cell_down: Key::Char('J'),
            extend_selection_by_one_cell_up: Key::Char('K'),
            tab_records: Key::Char('1'),
            tab_properties: Key::Char('2'),
            tab_sql_editor: Key::Char('3'),
            tab_columns: Key::Char('4'),
            tab_constraints: Key::Char('5'),
            tab_foreign_keys: Key::Char('6'),
            tab_indexes: Key::Char('7'),
            extend_or_shorten_widget_width_to_right: Key::Char('>'),
            extend_or_shorten_widget_width_to_left: Key::Char('<'),
            expand_column_width: Key::Char('='),
            shorten_column_width: Key::Char('-'),
            // reset_column_width: [Key::Char('0'), Key::Char('0')],
            delete: Key::Char('D'),
            edit_cell: Key::Char('C'),
            edit_command: Key::Char(':'),
            orderby_asc: Key::Char('o'),
            orderby_desc: Key::Char('O'),
            jump_to_start: Key::Char('0'),
            jump_to_end: Key::Char('$'),
            forward: Key::Char('f'),
            backward: Key::Char('F'),
            repeat_movement: Key::Char(';'),
        }
    }
}

impl Config {
    pub fn new(cli_config: &CliConfig) -> anyhow::Result<Self> {
        let config_path = if let Some(config_path) = &cli_config.config {
            config_path.clone()
        } else {
            get_app_config_path()?.join("config.toml")
        };
        if let Ok(file) = File::open(config_path) {
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents)?;

            let config: Result<Config, toml::de::Error> = toml::from_str(&contents);
            match config {
                Ok(config) => return Ok(config),
                Err(e) => panic!("fail to parse config file: {}", e),
            }
        }
        Ok(Config::default())
    }
}

impl Connection {

    pub fn new(db_url: &str) -> anyhow::Result<Self> {
        let u = Url::parse(&db_url)?;
        let db_type = match u.scheme() {
            "mysql" => DatabaseType::MySql,
            "postgres" => DatabaseType::Postgres,
            "sqlite" => DatabaseType::Sqlite,
            _ => anyhow::bail!("not supported {}", db_url),
        };
        let database = u.path_segments().map(|c| c.collect::<Vec<_>>().iter().map(|c| c.to_string()).next()).flatten();
        let pairs = u.query_pairs();
        let ssl_mode = pairs.filter(|p| p.0 == "ssl-mode").map(|p| p.1.to_string()).filter(|s| s.to_uppercase() != "DISABLED" && s.to_uppercase() != "DISABLE").next();
        let c = Self {
            r#type: db_type,
            name: None,
            user: Some(u.username().to_string()),
            host: u.host_str().map(|s| s.to_string()),
            port: u.port(),
            path: None,
            password: u.password().map(|s| s.to_string()),
            database: database,
            ssl_mode: ssl_mode,
            database_url: Some(db_url.to_string()),
        };
        return Ok(c)
    }
    
    pub fn database_url(&self) -> anyhow::Result<String> {
        if self.database_url.is_some() {
            return Ok(self.database_url.as_ref().map(|u| u.to_string()).unwrap())
        }
        match self.r#type {
            DatabaseType::MySql => {
                let user = self
                    .user
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("type mysql needs the user field"))?;
                let host = self
                    .host
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("type mysql needs the host field"))?;
                let port = self
                    .port
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("type mysql needs the port field"))?;
                let password = self
                    .password
                    .as_ref()
                    .map_or(String::new(), |p| p.to_string());
                let ssl_mode = self.ssl_mode.as_deref().unwrap_or("DISABLED");

                match self.database.as_ref() {
                    Some(database) => Ok(format!(
                        "mysql://{user}:{password}@{host}:{port}/{database}?ssl-mode={ssl_mode}",
                        user = user,
                        password = password,
                        host = host,
                        port = port,
                        database = database,
                        ssl_mode = ssl_mode,
                    )),
                    None => Ok(format!(
                        "mysql://{user}:{password}@{host}:{port}?ssl-mode={ssl_mode}",
                        user = user,
                        password = password,
                        host = host,
                        port = port,
                        ssl_mode = ssl_mode, 
                    )),
                }
            }
            DatabaseType::Postgres => {
                let user = self
                    .user
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("type postgres needs the user field"))?;
                let host = self
                    .host
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("type postgres needs the host field"))?;
                let port = self
                    .port
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("type postgres needs the port field"))?;
                let password = self
                    .password
                    .as_ref()
                    .map_or(String::new(), |p| p.to_string());
                let sslmode = self.ssl_mode.as_deref().unwrap_or("disable");
                match self.database.as_ref() {
                    Some(database) => Ok(format!(
                        "postgres://{user}:{password}@{host}:{port}/{database}?sslmode={sslmode}",
                        user = user,
                        password = password,
                        host = host,
                        port = port,
                        database = database,
                        sslmode = sslmode,
                    )),
                    None => Ok(format!(
                        "postgres://{user}:{password}@{host}:{port}?sslmode={sslmode}",
                        user = user,
                        password = password,
                        host = host,
                        port = port,
                        sslmode = sslmode,
                    )),
                }
            }
            DatabaseType::Mssql => {
                let user = self
                    .user
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("type postgres needs the user field"))?;
                let host = self
                    .host
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("type postgres needs the host field"))?;
                let port = self
                    .port
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("type postgres needs the port field"))?;
                let password = self
                    .password
                    .as_ref()
                    .map_or(String::new(), |p| p.to_string());

                match self.database.as_ref() {
                    Some(database) => Ok(format!(
                        "mssql://{user}:{password}@{host}:{port}/{database}",
                        user = user,
                        password = password,
                        host = host,
                        port = port,
                        database = database
                    )),
                    None => Ok(format!(
                        "mssql://{user}:{password}@{host}:{port}",
                        user = user,
                        password = password,
                        host = host,
                        port = port,
                    )),
                }                
            }
            DatabaseType::Sqlite => {
                let path = self.path.as_ref().map_or(
                    Err(anyhow::anyhow!("type sqlite needs the path field")),
                    |path| {
                        expand_path(path).ok_or_else(|| anyhow::anyhow!("cannot expand file path"))
                    },
                )?;

                Ok(format!("sqlite://{path}", path = path.to_str().unwrap()))
            }
        }
    }

    pub fn database_url_with_name(&self) -> anyhow::Result<String> {
        let database_url = self.database_url()?;

        Ok(match &self.name {
            Some(name) => format!(
                "[{name}] {database_url}",
                name = name,
                database_url = database_url
            ),
            None => database_url,
        })
    }

    pub fn _is_mysql(&self) -> bool {
        matches!(self.r#type, DatabaseType::MySql)
    }

    pub fn _is_postgres(&self) -> bool {
        matches!(self.r#type, DatabaseType::Postgres)
    }

    pub fn _is_mssql(&self) -> bool {
        matches!(self.r#type, DatabaseType::Mssql)
    }

    pub fn get_type(&self) -> DatabaseType {
        self.r#type.clone()
    }
}

pub fn get_app_config_path() -> anyhow::Result<std::path::PathBuf> {
    let mut path = if cfg!(target_os = "macos") {
        dirs_next::home_dir().map(|h| h.join(".config"))
    } else {
        dirs_next::config_dir()
    }
    .ok_or_else(|| anyhow::anyhow!("failed to find os config dir."))?;

    path.push("gobang");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

fn expand_path(path: &Path) -> Option<PathBuf> {
    let mut expanded_path = PathBuf::new();
    let mut path_iter = path.iter();
    if path.starts_with("~") {
        path_iter.next()?;
        expanded_path = expanded_path.join(dirs_next::home_dir()?);
    }
    for path in path_iter {
        let path = path.to_str()?;
        expanded_path = if cfg!(unix) && path.starts_with('$') {
            expanded_path.join(std::env::var(path.strip_prefix('$')?).unwrap_or_default())
        } else if cfg!(windows) && path.starts_with('%') && path.ends_with('%') {
            expanded_path
                .join(std::env::var(path.strip_prefix('%')?.strip_suffix('%')?).unwrap_or_default())
        } else {
            expanded_path.join(path)
        }
    }
    Some(expanded_path)
}

#[cfg(test)]
mod test {
    use super::{expand_path, KeyConfig, Path, PathBuf, Connection};
    use serde_json::Value;
    use std::env;

    #[test]
    fn test_overlappted_key() {
        let value: Value =
            serde_json::from_str(&serde_json::to_string(&KeyConfig::default()).unwrap()).unwrap();
        if let Value::Object(map) = value {
            let mut values: Vec<String> = map
                .values()
                .map(|v| match v {
                    Value::Object(map) => Some(format!("{:?}", map)),
                    _ => None,
                })
                .flatten()
                .collect();
            values.sort();
            let before_values = values.clone();
            values.dedup();
            pretty_assertions::assert_eq!(before_values, values);
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_expand_path() {
        let home = env::var("HOME").unwrap();
        let test_env = "baz";
        env::set_var("TEST", test_env);

        assert_eq!(
            expand_path(&Path::new("$HOME/foo")),
            Some(PathBuf::from(&home).join("foo"))
        );

        assert_eq!(
            expand_path(&Path::new("$HOME/foo/$TEST/bar")),
            Some(PathBuf::from(&home).join("foo").join(test_env).join("bar"))
        );

        assert_eq!(
            expand_path(&Path::new("~/foo")),
            Some(PathBuf::from(&home).join("foo"))
        );

        assert_eq!(
            expand_path(&Path::new("~/foo/~/bar")),
            Some(PathBuf::from(&home).join("foo").join("~").join("bar"))
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_expand_patha() {
        let home = std::env::var("HOMEPATH").unwrap();
        let test_env = "baz";
        env::set_var("TEST", test_env);

        assert_eq!(
            expand_path(&Path::new("%HOMEPATH%/foo")),
            Some(PathBuf::from(&home).join("foo"))
        );

        assert_eq!(
            expand_path(&Path::new("%HOMEPATH%/foo/%TEST%/bar")),
            Some(PathBuf::from(&home).join("foo").join(test_env).join("bar"))
        );

        assert_eq!(
            expand_path(&Path::new("~/foo")),
            Some(PathBuf::from(&dirs_next::home_dir().unwrap()).join("foo"))
        );

        assert_eq!(
            expand_path(&Path::new("~/foo/~/bar")),
            Some(
                PathBuf::from(&dirs_next::home_dir().unwrap())
                    .join("foo")
                    .join("~")
                    .join("bar")
            )
        );
    }

    #[test]
    fn test_new_connection() {
        let db_url = "postgres://root:123456@192.168.0.123:5432/test";
        let conn = Connection::new(db_url).unwrap();
        assert_eq!(conn.user, Some("root".to_string()));
        assert_eq!(conn.host, Some("192.168.0.123".to_string()));
        assert_eq!(conn.port, Some(5432));
        assert_eq!(conn.password, Some("123456".to_string()));
        assert_eq!(conn.database, Some("test".to_string()));
        assert_eq!(conn.database_url, Some(db_url.to_string()));
    }
}
