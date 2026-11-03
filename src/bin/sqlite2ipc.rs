use std::io;
use std::process::ExitCode;

use rs_sqlite2arrow_ipc::BAT_SIZE_DEFAULT;
use rs_sqlite2arrow_ipc::Config;

fn io_envkey2val(key: &'static str) -> impl FnMut() -> String {
    move || std::env::var(key).unwrap_or_default()
}

fn string2bool(s: String) -> bool {
    str::parse(&s).unwrap_or_default()
}

fn string2usize(s: String) -> Result<usize, io::Error> {
    str::parse(&s).map_err(io::Error::other)
}

fn io_select_all() -> impl FnMut() -> String {
    io_envkey2val("ENV_SQL_SELECT_ALL")
}

fn io_select_one() -> impl FnMut() -> String {
    io_envkey2val("ENV_SQL_SELECT_ONE")
}

fn io_allow_null() -> impl FnMut() -> bool {
    || {
        let s: String = io_envkey2val("ENV_ALLOW_NULL")();
        string2bool(s)
    }
}

fn io_bat_size() -> impl FnMut() -> usize {
    || {
        let s: String = io_envkey2val("ENV_BAT_SIZE")();
        string2usize(s).unwrap_or(BAT_SIZE_DEFAULT)
    }
}

fn io_path2sqlite() -> impl FnMut() -> String {
    io_envkey2val("ENV_SQLITE_DB_FILE_PATH")
}

fn io_config() -> impl FnMut() -> Config {
    || {
        let select_all: String = io_select_all()();
        let sel1: String = io_select_one()();
        let select1: Option<String> = match sel1.is_empty() {
            true => None,
            false => Some(sel1),
        };
        let nullable: bool = io_allow_null()();
        let bat_size: usize = io_bat_size()();
        let path2sqlite: String = io_path2sqlite()();
        Config {
            select_all,
            select1,
            nullable,
            bat_size,
            path2sqlite,
        }
    }
}

fn io_main() -> impl FnOnce() -> Result<(), io::Error> {
    || {
        let cfg: Config = io_config()();
        cfg.sqlite2stdout()
    }
}

fn sub() -> Result<(), io::Error> {
    io_main()()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");

        let cfg: Config = io_config()();

        let Config {
            select_all,
            select1,
            nullable,
            bat_size,
            path2sqlite,
        } = cfg;

        eprintln!("ENV_SQL_SELECT_ALL: {select_all}");
        eprintln!("ENV_SQL_SELECT_ONE: {}", select1.unwrap_or_default());
        eprintln!("ENV_ALLOW_NULL: {nullable}");
        eprintln!("ENV_BAT_SIZE: {bat_size}");
        eprintln!("ENV_SQLITE_DB_FILE_PATH: {path2sqlite}");

        ExitCode::FAILURE
    })
}
