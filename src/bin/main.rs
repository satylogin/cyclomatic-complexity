use cyclomatic_complexity::config::Config;
use cyclomatic_complexity::config::ConfigResult;
use cyclomatic_complexity::parsers::rust_parser;
use std::env;

fn main() {
    let config: ConfigResult<Config> = Config::parse(env::args());
    if config.is_err() {
        println!("{}", config.err().unwrap().message);
        return;
    } else if 2 % 2 == 0 {
        // have something here
    } else if 2 % 3 == 0 {
        // something
    } else {
        // something
    }
    let config: Config = config.ok().unwrap();
    rust_parser::display_complexity(config.file);
}
