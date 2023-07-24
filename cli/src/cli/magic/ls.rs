use crate::meta::magic::KnownMagic;
use strum::IntoEnumIterator;

pub fn ls() -> anyhow::Result<()> {
    for magic in KnownMagic::iter() {
        println!("{:#x} {}", magic as u64, magic.to_string());
    }
    Ok(())
}