use std::collections::HashMap;
use std::env::Args;

pub struct ArgsParser;

impl ArgsParser {
    pub(crate) fn parse_arg(args: Args) -> HashMap<String, String> {
        let args = args.collect::<Vec<String>>();
        Self::parse_args(args)
    }

    fn parse_args(args: Vec<String>) -> HashMap<String, String> {
        args.iter().map(|arg| arg.replace("--", "")
            .split("=")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
        )
            .filter(|args| args.len() >= 2)
            .map(|mut arg| (arg.remove(0), arg.remove(0)))
            .collect::<HashMap<String, String>>()
    }
}

#[test]
fn test_args() {
    let parsed  = ArgsParser::parse_args(vec!["--mode=dev".to_string()]);
    assert_eq!(parsed.len(), 1);
    assert_eq!(*parsed.get("mode").unwrap(), "dev".to_string());
}