use clap::{self, App, Arg, ArgMatches};
use std::ffi::OsString;
use std::result::Result;

const APP_NAME: &str = "CYCLOMATIC COMPLEXITY";
const VERSION: &str = "0.1";
const ABOUT: &str = "This CLI find the cyclomatic complexity associated with the file";

#[derive(Debug)]
pub struct Config {
    file: String,
}

pub type ConfigResult<T> = Result<T, clap::Error>;

impl Config {
    pub fn parse<I, T>(iter: I) -> ConfigResult<Config>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let args: ArgMatches = parse(iter)?;

        Ok(Config {
            file: args.value_of("file").unwrap().to_string(),
        })
    }
}

fn parse<I, T>(iter: I) -> clap::Result<ArgMatches<'static>>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    App::new(APP_NAME)
        .version(VERSION)
        .about(ABOUT)
        .arg(
            Arg::with_name("file")
                .help("file name to check cyclomatic complixity for")
                .long("file")
                .required(true)
                .takes_value(true),
        )
        .get_matches_from_safe(iter)
}

#[cfg(test)]
mod tests {
    use super::Config;
    use rstest::rstest;

    #[test]
    fn valid_args() {
        let args = vec!["prog", "--file", "test_file"];
        let config: Config = Config::parse(args).ok().unwrap();
        assert_eq!("test_file", config.file);
    }

    #[rstest]
    #[case(vec!["prog", "--file"])]
    #[case(vec!["prog"])]
    #[case(vec!["prog", "--alien", "ben10"])]
    fn invalid_args_test(#[case] input: Vec<&str>) {
        assert!(Config::parse(input).is_err());
    }
}
