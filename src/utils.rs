const USAGE_STR: &str = "Usage:
  - insert <key> <value>
  - get <key>
  - delete <key>
  - info
  - exit
";

pub fn print_usage() {
    println!("{USAGE_STR}")
}

const INFO_STR: &str = "Info: 
  - collision values: to simulate collisions without having to try for eons, there are
                      hard coded values that collide. Here is a list of them:
                      ['hello', 'food', 'meal', 'why', 'me', 'i'] => 1
";

pub fn print_info() {
    println!("{INFO_STR}")
}
