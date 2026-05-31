const USAGE_STR: &str = 
"Usage:
  - insert <key> <value>
  - get <value>
  - info
  - exit

Note that <key> is a string and <value> is an unsigned integer*.
*meaning that <value> is the offset to the entry that <key> is associated with
";

pub fn print_usage() {
    println!("{USAGE_STR}")
}


const INFO_STR: &str = 
"Info: 
  - collision values: to simulate collisions without having to try for eons, there are
                      hard coded values that collide. Here is a list of them:
                      ['hello', 'food', 'why', 'me', 'i'] => 1
";

pub fn print_info() {
    println!("{INFO_STR}")
}

