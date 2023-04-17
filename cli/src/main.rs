pub struct NoLabel;

impl sid::Label for NoLabel {
    fn label() -> &'static str {
        ""
    }
}

use std::io::Read;
use clap::{Parser};
use sid::Label;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    uuid: Option<String>,
}

fn main() {
    let args = Cli::parse();
    if let Some(uuid) = args.uuid {
        let uuid = uuid::Uuid::parse_str(&uuid).unwrap();
        let sid = sid::Sid::<NoLabel>::from(uuid);
        println!("{}", sid);
        //check if has stdin with atty
    } else if !atty::is(atty::Stream::Stdin) {
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer).unwrap();
        let uuid = uuid::Uuid::parse_str(&buffer).unwrap();
        let sid = sid::Sid::<NoLabel>::from(uuid);
        println!("{}", sid);
    } else {
        let sid = NoLabel::sid();
        println!("{}", sid);
    }
}
