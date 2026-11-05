use crate::{
    db::{DB, error::ExecuteError},
    utils::{print_info, print_usage},
};

const SEP: char = '\0';

pub fn execute<'a>(buf: &'a str, db: &'a mut DB) -> Result<(), ExecuteError<'a>> {
    // TODO: preprocess buf to enable multiline strings
    match pre_process(buf).split(SEP).collect::<Vec<_>>().as_slice() {
        [""] => (),
        ["insert", key, val] => {
            println!("Inserting value '{key}', '{val}' to entry-file");
            db.insert(key, val)?;
        }
        ["get", s] => {
            println!("getting value '{s}'");
            println!("Value found: {}", db.get(s)?);
        }
        ["delete", key] => {
            println!("Deleting value '{key}'");
            db.delete(key)?;
        }
        ["update", key, val] => {
            println!("Updating value '{key}' to '{val}'");
            db.update(key, val)?;
        }
        ["info"] => print_info(),
        ["hash", key] => db.print_hash(key),
        ["free"] => db.print_free_list(),
        ["de", de_id] => db.print_de(de_id),
        ["state"] => db.print_state(),
        ["exit"] => return Err(ExecuteError::ExitCmd),
        _ => print_usage(),
    };
    Ok(())
}

fn pre_process(buf: &str) -> String {
    let mut v = vec![];
    let s = buf.trim();
    let mut quote = false;
    for c in s.chars() {
        match c {
            '"' => quote = !quote,
            s if s == ' ' && !quote => v.push(SEP),
            s if s == '\n' && !quote => (),
            _ => v.push(c),
        }
    }
    v.iter().collect()
}
