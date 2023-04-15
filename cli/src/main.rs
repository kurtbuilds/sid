pub struct NoLabel;

impl oid::Label for NoLabel {
    fn label() -> &'static str {
        ""
    }
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let uuid = args.into_iter().skip(1).next().unwrap();
    let uuid = uuid::Uuid::parse_str(&uuid).unwrap();
    let oid = oid::Oid::<NoLabel>::from(uuid);
    println!("{}", oid);

}
