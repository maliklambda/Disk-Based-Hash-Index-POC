use crate::{index::{
    Index,
    error::{ExecuteError, InsertErr},
}, utils::{print_info, print_usage}};



const SEP: char = '\0';

pub fn execute(buf: &str, idx: &mut Index) -> Result<(), ExecuteError> {
    // TODO: preprocess buf to enable multiline strings
    match pre_process(buf).split(SEP).collect::<Vec<_>>().as_slice() {
        [""] => (),
        ["insert", s, v] => {
            println!("Inserting value '{s}', '{v}'");
            let val: u64 = v.parse().map_err(InsertErr::ParseErr)?;
            idx.insert(s, val)?;
        }
        ["get", s] => {
            println!("getting value '{s}'");
            println!("Value found: {}", idx.get(s)?);
        }
        ["info"] => print_info(),
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
