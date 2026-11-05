use crate::db::hash::HashAlgorithm;

#[derive(Debug)]
pub struct Config {
    pub hash_algorithm: HashAlgorithm,
    pub entries_fname: String,
    pub content_fname: String,
}

pub fn parse_cli_args() -> Result<Config, String> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    let mut idx = 0;
    // initialize fields with none values
    let mut hash_algorithm = None;
    let mut content_fname = None;
    let mut entries_fname = None;
    println!("ARGS: {:?}", args);
    loop {
        if args.len() <= idx {
            break;
        }
        match args[idx].as_str() {
            "--content" => {
                idx += 1;
                if args.len() <= idx {
                    return Err(format!("Missing specifier for cli-arg: {}.", args[idx - 1]));
                }
                content_fname = Some(args[idx].clone());
                idx += 1;
            }
            "--entries" => {
                idx += 1;
                if args.len() <= idx {
                    return Err(format!("Missing specifier for cli-arg: {}.", args[idx - 1]));
                }
                entries_fname = Some(args[idx].clone());
                idx += 1;
            }
            "--hash" => {
                idx += 1;
                if args.len() <= idx {
                    return Err(format!("Missing specifier for cli-arg: {}.", args[idx - 1]));
                }
                // Match on supported hash-algorithms
                hash_algorithm = Some(match args[idx].as_str() {
                    "xxh64" => HashAlgorithm::XXHash64,
                    "default" => HashAlgorithm::Default,
                    _ => {
                        return Err(format!("Unsupported hash algorithm: {}.", args[idx]));
                    }
                });
                // hash_algorithm = Some(args[idx]);
                idx += 1;
            }
            _ => return Err(format!("Invalid cli-arg: {}.", args[idx])),
        }
    }
    Ok(Config {
        entries_fname: entries_fname.unwrap_or("entries.db".to_string()),
        content_fname: content_fname.unwrap_or("content.db".to_string()),
        hash_algorithm: hash_algorithm.unwrap_or(HashAlgorithm::Default),
    })
}

pub const CLI_ARGS_INFO_STR: &str = "Cli Args Info: 
  --content <content-file-name>
  --entries <entries-file-name>
  --hash <'default', 'xxh64>
";
