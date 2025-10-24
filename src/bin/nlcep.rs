use nlcep::NewEvent;

use std::env;

fn main() {
    let args_without_path: Vec<_> = env::args().skip(1).collect();
    let input = args_without_path.join(" ");
    let event = input.parse::<NewEvent>();
    println!("{:?}", event);
}
