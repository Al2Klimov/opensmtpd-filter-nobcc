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

#[cfg(test)]
mod tests {
    use super::*;
    use mail_parser::MessageParser;

    #[test]
    fn join_write_bytes_empty_input() {
        let mut buf = Vec::<u8>::new();
        join_write_bytes(&mut buf, b"|", std::iter::empty()).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn join_write_bytes_single_part() {
        let mut buf = Vec::<u8>::new();
        join_write_bytes(&mut buf, b"|", [b"hello".as_slice()].into_iter()).unwrap();
        assert_eq!(buf, b"hello");
    }

    #[test]
    fn join_write_bytes_multiple_parts() {
        let mut buf = Vec::<u8>::new();
        join_write_bytes(
            &mut buf,
            b"|",
            [b"a".as_slice(), b"b".as_slice(), b"c".as_slice()].into_iter(),
        )
        .unwrap();
        assert_eq!(buf, b"a|b|c");
    }

    #[test]
    fn has_address_returns_false_for_none() {
        assert!(!has_address(None, &"test@example.com".to_string()));
    }

    #[test]
    fn has_address_returns_true_for_matching_address() {
        let mail = MessageParser::new()
            .parse_headers(b"To: test@example.com\r\n\r\n")
            .unwrap();
        assert!(has_address(mail.to(), &"test@example.com".to_string()));
    }

    #[test]
    fn has_address_returns_false_for_non_matching_address() {
        let mail = MessageParser::new()
            .parse_headers(b"To: other@example.com\r\n\r\n")
            .unwrap();
        assert!(!has_address(mail.to(), &"test@example.com".to_string()));
    }
}
