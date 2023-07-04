use std::io::{IsTerminal, Read};
use std::str::FromStr;
use clap::{Parser};
use sid::{Label, NoLabel, Sid};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    id: Option<String>,
}
0123456789abcdefghjkmnpqrstvwxyz

fn main() {
    let args = Cli::parse();

    let input_id = args.id.or_else(|| {
        let mut buffer = String::new();
        let mut stdin = std::io::stdin();
        if stdin.is_terminal() {
            None
        } else {
            stdin.read_to_string(&mut buffer).unwrap();
            Some(buffer.trim().to_string())
        }
    });
    if let Some(id) = input_id {
        if id.len() == 27 {
            // it's a sid. convert it to uuid
            let sid: Sid = Sid::from_str(&id).unwrap();
            let uuid = sid.uuid();
            println!("{}", uuid);
        } else if id.len() == 36 {
            // it's a uuid. convert it to sid
            let uuid = uuid::Uuid::parse_str(&id).unwrap();
            let sid: Sid = Sid::from(uuid);
            println!("{}", sid);
        } else {
            println!("Expected a uuid (len 36) or a sid (len 27). Input was: {}", id);
        }
    } else {
        let sid = NoLabel::sid();
        println!("{}", sid);
    }
}