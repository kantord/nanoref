use crate::checksum::{check_chars, SALT_V1};
use anyhow::Result;
use nanoid::nanoid;

pub fn run() -> Result<()> {
    let body = nanoid!();
    let (c1, c2) = check_chars(SALT_V1, &body);
    println!("nref-{}{}{}", c1, c2, body);
    Ok(())
}
