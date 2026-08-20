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
    let mut idx = DB::init("index.db", "entries.db").unwrap();
    let mut buf = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buf).unwrap();
        match execute(&buf, &mut idx) {
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
