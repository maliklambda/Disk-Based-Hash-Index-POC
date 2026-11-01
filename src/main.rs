use std::io::{self, Write};

use crate::{
    db::{
        DB,
        error::{ExecuteError, GetErr},
    },
    execute::execute,
};

pub mod db;
pub mod execute;
pub mod utils;

fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();
    let mut db = DB::init("entries.db", "content.db").unwrap();
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
    db.insert("why", "aa").unwrap();
    db.insert("me", "bbbb").unwrap();
    db.delete("me").unwrap();
}
