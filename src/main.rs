use itertools::Itertools;
use mail_parser::{Address, MessageParser};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::io::{Write, stderr, stdin, stdout};

fn main() -> std::io::Result<()> {
    let mut std_out = stdout().lock();
    let mut std_err = stderr().lock();
    let mut sessions = HashMap::<String, (Vec<String>, String)>::new();

    for l in stdin().lines() {
        let line = l?;
        let mut fields = line.split("|");

        match fields.next() {
            Some("config") => match fields.next() {
                Some("ready") => {
                    writeln!(std_out, "register|report|smtp-in|tx-begin")?;
                    writeln!(std_out, "register|report|smtp-in|tx-rcpt")?;
                    writeln!(std_out, "register|filter|smtp-in|data-line")?;
                    writeln!(std_out, "register|filter|smtp-in|commit")?;
                    writeln!(std_out, "register|report|smtp-in|link-disconnect")?;
                    writeln!(std_out, "register|ready")?;
                }
                _ => {}
            },
            Some("report") => {
                fields.next(); // protocol version
                fields.next(); // timestamp
                fields.next(); // subsystem

                match (fields.next(), fields.next()) {
                    (Some(phase), Some(session)) => match phase {
                        "tx-begin" => {
                            sessions.insert(session.to_owned(), Default::default());
                        }
                        "tx-rcpt" => match (fields.next(), fields.next(), fields.next()) {
                            (Some(_), Some("ok"), Some(rcpt)) => match sessions.get_mut(session) {
                                None => {}
                                Some((rcpts, _)) => rcpts.push(rcpt.to_owned()),
                            },
                            _ => {}
                        },
                        "link-disconnect" => {
                            sessions.remove(session);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Some("filter") => {
                fields.next(); // protocol version
                fields.next(); // timestamp
                fields.next(); // subsystem

                match (fields.next(), fields.next(), fields.next()) {
                    (Some(phase), Some(session), Some(token)) => match phase {
                        "data-line" => {
                            writeln!(
                                std_out,
                                "filter-dataline|{}|{}|{}",
                                session,
                                token,
                                fields.clone().format("|")
                            )?;

                            let mut flds = fields.clone();

                            match (flds.next(), flds.next()) {
                                (Some("."), None) => {}
                                _ => match sessions.get_mut(session) {
                                    None => {}
                                    Some((_, mail)) => {
                                        writeln!(mail, "{}", fields.format("|")).unwrap();
                                    }
                                },
                            }
                        }
                        "commit" => {
                            writeln!(
                                std_out,
                                "filter-result|{}|{}|{}",
                                session,
                                token,
                                if match sessions.get(session) {
                                    None => true,
                                    Some((rcpts, mail)) =>
                                        match MessageParser::new().parse_headers(mail) {
                                            None => {
                                                writeln!(std_err, "Malformed eMail:")?;
                                                write!(std_err, "{}", mail)?;
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

    Ok(())
}

fn has_address(haystack: Option<&Address>, needle: &String) -> bool {
    match haystack {
        None => false,
        Some(hs) => hs.contains(needle),
    }
}
