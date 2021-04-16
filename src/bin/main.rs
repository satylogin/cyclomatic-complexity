use cyclomatic_complexity::config::Config;
use std::env;

fn main() {
    let config: Config = Config::parse(env::args()).ok().unwrap();
    println!("{:?}", config);

    // let complexity = calculator::calculate(config.file, ASTGraphParser);
    // println!("calculated complexity: {}", complexity);
}
