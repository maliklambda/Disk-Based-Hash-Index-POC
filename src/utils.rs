const USAGE_STR: &str = "Usage:
  - insert <key> <value>
  - get <key>
  - delete <key>
  - info
  - exit
  - hash
";

pub fn print_usage() {
    println!("{USAGE_STR}")
}

const INFO_STR: &str = "Info: 
  - Collision Values: To simulate collisions without having to try for eons, there are
                      hard coded values that collide. Here is a list of them:
                      ['hello', 'food', 'meal', 'why', 'me', 'i'] => 1
  - Quotation Marks:  Single words, without a SPACE (' ') character are allowed as inputs.
                      However, quotation marks ('\"') are required when the string contains 
                      at least one SPACE (' ') character. Quotation marks may also be used 
                      for single word inputs.
  - New Lines:        New lines may be used freely in the input as newline-characters ('\\n').
                      They do not require quotation marks or any special treatment. However,
                      multiline editing inside the CLI is not supported as of now.
";

pub fn print_info() {
    println!("{INFO_STR}")
}

