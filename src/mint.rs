use crate::checksum::{check_chars, SALT_V1};
use nanoid::nanoid;

pub fn run(json: bool) {
    let body = nanoid!();
    let (c1, c2) = check_chars(SALT_V1, &body);
    let marker = format!("nref-{c1}{c2}{body}");
    if json {
        println!("{}", serde_json::json!({"marker": marker}));
    } else {
        println!("{marker}");
    }
}
