mod util;

use mail_parser::MessageParser;
use std::collections::HashMap;
use std::io::{BufRead, Write, stderr, stdin, stdout};
use util::{has_address, join_write_bytes};

fn main() -> std::io::Result<()> {
    let mut std_in = stdin().lock();
    let mut std_out = stdout().lock();
    let mut std_err = stderr().lock();

    let mut line = Vec::<u8>::new();
    let mut sessions = HashMap::<Vec<u8>, (Vec<String>, Vec<u8>)>::new();

    loop {
        line.clear();
        std_in.read_until(b'\n', &mut line)?;

        while line
            .pop_if(|last| match last {
                b'\r' => true,
                b'\n' => true,
                _ => false,
            })
            .is_some()
        {}

        let mut fields = line.split(|&sep| sep == b'|');

        match fields.next() {
            Some(b"config") => match fields.next() {
                Some(b"ready") => {
                    writeln!(std_out, "register|report|smtp-in|tx-begin")?;
                    writeln!(std_out, "register|report|smtp-in|tx-rcpt")?;
                    writeln!(std_out, "register|filter|smtp-in|data-line")?;
                    writeln!(std_out, "register|filter|smtp-in|commit")?;
                    writeln!(std_out, "register|report|smtp-in|link-disconnect")?;
                    writeln!(std_out, "register|ready")?;
                }
                _ => {}
            },
            Some(b"report") => {
                fields.next(); // protocol version
                fields.next(); // timestamp
                fields.next(); // subsystem

                match (fields.next(), fields.next()) {
                    (Some(phase), Some(session)) => match phase {
                        b"tx-begin" => {
                            sessions.insert(session.to_owned(), Default::default());
                        }
                        b"tx-rcpt" => match (fields.next(), fields.next(), fields.next()) {
                            (Some(_), Some(b"ok"), Some(rcpt)) => match sessions.get_mut(session) {
                                None => {}
                                Some((rcpts, _)) => match String::from_utf8(rcpt.to_owned()) {
                                    Err(_) => {}
                                    Ok(rcpt) => rcpts.push(rcpt),
                                },
                            },
                            _ => {}
                        },
                        b"link-disconnect" => {
                            sessions.remove(session);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Some(b"filter") => {
                fields.next(); // protocol version
                fields.next(); // timestamp
                fields.next(); // subsystem

                match (fields.next(), fields.next(), fields.next()) {
                    (Some(phase), Some(session), Some(token)) => match phase {
                        b"data-line" => {
                            std_out.write_all(b"filter-dataline|")?;
                            std_out.write_all(session)?;
                            std_out.write_all(b"|")?;
                            std_out.write_all(token)?;
                            std_out.write_all(b"|")?;

                            join_write_bytes(&mut std_out, b"|", fields.clone())?;
                            writeln!(std_out, "")?;

                            let mut flds = fields.clone();

                            match (flds.next(), flds.next()) {
                                (Some(b"."), None) => {}
                                _ => match sessions.get_mut(session) {
                                    None => {}
                                    Some((_, mail)) => {
                                        join_write_bytes(mail, b"|", fields)?;
                                        writeln!(mail, "")?;
                                    }
                                },
                            }
                        }
                        b"commit" => {
                            std_out.write_all(b"filter-result|")?;
                            std_out.write_all(session)?;
                            std_out.write_all(b"|")?;
                            std_out.write_all(token)?;

                            writeln!(
                                std_out,
                                "|{}",
                                if match sessions.get(session) {
                                    None => true,
                                    Some((rcpts, mail)) =>
                                        match MessageParser::new().parse_headers(mail) {
                                            None => {
                                                writeln!(std_err, "Malformed eMail:")?;
                                                std_err.write_all(mail)?;
                                                writeln!(std_err, ".")?;
                                                true
                                            }
                                            Some(mail) => {
                                                let mut allow = true;

                                                for rcpt in rcpts {
                                                    if !has_address(mail.to(), rcpt)
                                                        && !has_address(mail.cc(), rcpt)
                                                    {
                                                        writeln!(
                                                            std_err,
                                                            "Missing in To/Cc: {}",
                                                            rcpt
                                                        )?;

                                                        allow = false;
                                                    }
                                                }

                                                allow
                                            }
                                        },
                                } {
                                    writeln!(std_err, "Allowing")?;
                                    "proceed"
                                } else {
                                    writeln!(std_err, "Denying")?;
                                    "reject|550 Forbidden"
                                }
                            )?;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
