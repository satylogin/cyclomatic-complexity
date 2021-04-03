use cyclomatic_complexity::{config::Config, cyclomatic};
use std::env;

fn main() {
    let config: Config = Config::parse(env::args()).ok().unwrap();
    println!("{:?}", config);

    let complexity = cyclomatic::calculate_complexity(config.file);
    println!("calculated complexity: {}", complexity);
}
