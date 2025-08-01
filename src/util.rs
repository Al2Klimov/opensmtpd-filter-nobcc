use mail_parser::Address;
use std::io::{Result, Write};

pub(crate) fn join_write_bytes<'a>(
    writer: &mut dyn Write,
    sep: &[u8],
    mut parts: impl Iterator<Item = &'a [u8]>,
) -> Result<()> {
    match parts.next() {
        None => {}
        Some(first) => {
            writer.write_all(first)?;

            for part in parts {
                writer.write_all(sep)?;
                writer.write_all(part)?;
            }
        }
    }

    Ok(())
}

pub(crate) fn has_address(haystack: Option<&Address>, needle: &String) -> bool {
    match haystack {
        None => false,
        Some(hs) => hs.contains(needle),
    }
}
