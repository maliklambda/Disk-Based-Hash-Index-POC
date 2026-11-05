use std::io::{self, Write};

use crate::{
    config::{parse_cli_args, CLI_ARGS_INFO_STR},
    db::{
        DB,
        error::{ExecuteError, GetErr},
    },
    execute::execute,
};

mod config;
mod db;
mod execute;
mod utils;

fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();
    let config = parse_cli_args().unwrap_or_else(|_| panic!("{CLI_ARGS_INFO_STR}"));
    let mut db = DB::init(config).unwrap();
    println!("DB: {:?}", db);
    let mut buf = String::new();
    start_up_state(&mut db);
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buf).unwrap();
        match execute(&buf, &mut db) {
            Ok(()) => {}
            Err(ExecuteError::ExitCmd) => break,
            Err(ExecuteError::GetErr(GetErr::HashNotFound(h))) => {
                println!("Value not found (hash = {h})")
            }
            Err(err) => println!("Error: {:?}", err),
        }
        buf.clear();
    }
    println!("Bye bye.");
}

/// Create this state upon startup.
fn start_up_state(db: &mut DB) {
    db.insert("why", "old").unwrap();
    db.update("why", "new").unwrap();
}
