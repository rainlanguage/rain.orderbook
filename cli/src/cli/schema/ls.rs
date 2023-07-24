use crate::meta::KnownMeta;
use strum::IntoEnumIterator;

pub fn ls() -> anyhow::Result<()> {
    for schema in KnownMeta::iter() {
        println!("{}", schema);
    }
    Ok(())
}