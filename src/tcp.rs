use base64::engine::general_purpose::STANDARD;
use base64::read::DecoderReader;
use base64::write::EncoderWriter;
use std::error::Error;
use std::io::{Read, Write};
use tokio::net::TcpStream;

pub struct Connect {
    stream: TcpStream,
}

impl Connect {
    pub async fn new() -> Self {
        let stream = TcpStream::connect("127.0.0.1:6123").await.unwrap();
        Connect { stream }
    }

    fn get(&self) -> &TcpStream {
        &self.stream
    }

    pub async fn request(&self, action: Action) -> Result<Ack, Box<dyn Error>> {
        let stream = self.get();
        write_request(stream, action).await?;
        read_response(stream).await
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    CheckIdentity {
        password: String,
    },
    // user_account
    GetInfo,
    // website_account
    // GetWebsiteAccountPassword {
    //     website_id: i32,
    // },
    AddWebsiteAccount {
        account: String,
        password: String,
        site_url: String,
        site_name: Option<String>,
        note: Option<String>,
    },
    ChangeWebsiteAccount {
        id: i32,
        new_account: String,
        new_password: String,
        new_site_name: Option<String>,
        new_site_url: String,
        new_note: Option<String>,
    },
    DeleteWebsiteAccount {
        website_id: i32,
    },
    // check_dead_link
    CheckDeadLink,
}

#[derive(Debug)]
pub struct AccountListItem {
    pub id: Option<i32>,
    pub account: String,
    pub password: String,
    pub site_url: String,
    pub site_name: Option<String>,
    pub note: Option<String>,
    pub is_dead: bool,
}

#[derive(Debug, Default)]
pub struct AccountList {
    pub list: Vec<AccountListItem>,
    pub selected: usize,
}

/// #[derive(Debug)]
/// pub struct AccountListItem {
///     pub id: Option<i32>,
///     pub account: String,
///     pub password: String,
///     pub site_url: String,
///     pub site_name: Option<String>,
///     pub note: Option<String>,
/// }
#[derive(Debug)]
pub enum Ack {
    Ack,
    Info { list: Vec<AccountListItem> },
    DeadLink { list: Vec<(i32, bool)> },
    IdentityError,
    DbError,
}

/// read the ack from the socket and return a task
///
/// Here is the TCP format:
/// "ACTION\tOTHER_MESSAGE"
///
/// for example:
/// - `"0"`
/// - `"1\nmy_account\tmy_password\tmy_site_url\tmy_site_name\tmy_note"`
///
/// ## Here is the list of action:
/// > - 0: Ack
/// > - 1: Info
/// > - 2: DeadLink
/// > - 3: IdentityError
/// > - 4: DbError
///
pub async fn read_response(stream: &TcpStream) -> Result<Ack, Box<dyn Error>> {
    stream.readable().await?;
    let mut buffer = [0; 4096];
    match stream.try_read(&mut buffer) {
        Ok(0) => Err("Failed to read from socket".into()),
        Ok(_) => {
            let request = String::from_utf8_lossy(&buffer);
            let request = request.trim_end_matches('\0');
            let parts: Vec<&str> = request.split('\n').collect();
            Ok(pack_ack(parts)?)
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Err("Blocked".into()),
        Err(e) => Err(e.into()),
    }
}

fn pack_ack(parts: Vec<&str>) -> Result<Ack, Box<dyn Error>> {
    let action = parts[0].trim_end_matches('\0');
    let action = action.parse::<i32>()?;
    match action {
        0 => Ok(Ack::Ack),
        1 => {
            let mut list = Vec::new();
            for item in parts.iter().skip(1) {
                let item_parts: Vec<&str> = item.split('\t').collect();
                let id = item_parts[0].parse::<i32>()?;
                let account = item_parts[1].to_string();
                let password = item_parts[2].to_string();
                let site_url = item_parts[3].to_string();
                let site_name = if item_parts[4].is_empty() {
                    None
                } else {
                    Some(item_parts[4].to_string())
                };
                let note = if item_parts[5].is_empty() {
                    None
                } else {
                    Some(decode(item_parts[5].to_string()))
                };
                let is_dead = item_parts[6].parse::<i32>()? == 0;
                list.push(AccountListItem {
                    id: Some(id),
                    account,
                    password,
                    site_url,
                    site_name,
                    note,
                    is_dead,
                });
            }
            Ok(Ack::Info { list })
        }
        2 => {
            let mut list = Vec::new();
            for item in parts.iter().skip(1) {
                let item_parts: Vec<&str> = item.split('\t').collect();
                let id = item_parts[0].parse::<i32>()?;
                let is_dead = item_parts[1].parse::<i32>()?;
                list.push((id, is_dead == 1))
            }
            Ok(Ack::DeadLink { list })
        }
        3 => Ok(Ack::IdentityError),
        4 => Ok(Ack::DbError),
        _ => Err("Invalid ack".into()),
    }
}

/// write the request from the socket and return a task
///
/// Here is the TCP format:
/// "ACTION\tOTHER_MESSAGE"
///
/// for example:
/// - `"CheckIdentity\tmy_password"`
/// - `"AddWebsiteAccount\tmy_account\tmy_password\tmy_site_url\tmy_site_name\tmy_note"`
///
/// ## Here is the list of action:
/// > - 0: CheckIdentity
/// > - 1: Login
/// > - 2: AddWebsiteAccount
/// > - 3: ChangeWebsiteAccount
/// > - 4: DeleteWebsiteAccount
/// > - 5: CheckDeadLink
///
pub async fn write_request(stream: &TcpStream, action: Action) -> Result<(), Box<dyn Error>> {
    let response = depack_action(action);
    stream.writable().await?;
    match stream.try_write(response.as_bytes()) {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Err("Blocked".into()),
        Err(e) => Err(e.into()),
    }
}

fn depack_action(action: Action) -> String {
    match action {
        Action::CheckIdentity { password } => format!("0\t{}", password),
        Action::GetInfo => "1".to_string(),
        Action::AddWebsiteAccount {
            account,
            password,
            site_url,
            site_name,
            note,
        } => {
            let site_name = site_name.unwrap_or("".to_string());
            let note = note.unwrap_or("".to_string());
            format!(
                "2\t{}\t{}\t{}\t{}\t{}",
                account, password, site_url, site_name, encode(note)
            )
        }
        Action::ChangeWebsiteAccount {
            id,
            new_account,
            new_password,
            new_site_name,
            new_site_url,
            new_note,
        } => {
            let new_site_name = new_site_name.unwrap_or("".to_string());
            let new_note = new_note.unwrap_or("".to_string());
            format!(
                "3\t{}\t{}\t{}\t{}\t{}\t{}",
                id, new_account, new_password, new_site_name, new_site_url, encode(new_note)
            )
        }
        Action::DeleteWebsiteAccount { website_id } => {
            format!("4\t{}", website_id)
        }
        Action::CheckDeadLink => "5".to_string(),
    }
}

fn encode(data: String) -> String {
    let data = data.as_bytes();
    let mut encoder = EncoderWriter::new(Vec::new(), &STANDARD);
    encoder.write_all(data).unwrap();
    let str = encoder.finish().unwrap();
    String::from_utf8(str.clone()).unwrap()
}

fn decode(data: String) -> String {
    let mut decoder = DecoderReader::new(data.as_bytes(), &STANDARD);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded).unwrap();
    String::from_utf8(decoded).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depack_action() {
        assert_eq!(
            depack_action(Action::CheckIdentity {
                password: "my_password".to_string()
            }),
            "CheckIdentity\tmy_password"
        );
        assert_eq!(depack_action(Action::GetInfo), "Login");
        assert_eq!(
            depack_action(Action::AddWebsiteAccount {
                account: "my_account".to_string(),
                password: "my_password".to_string(),
                site_url: "my_site_url".to_string(),
                site_name: Some("my_site_name".to_string()),
                note: Some("my_note".to_string())
            }),
            "AddWebsiteAccount\tmy_account\tmy_password\tmy_site_url\tmy_site_name\tmy_note"
        );
        assert_eq!(
            depack_action(Action::ChangeWebsiteAccount {
                id: 1,
                new_account: "my_account".to_string(),
                new_password: "my_password".to_string(),
                new_site_name: Some("my_site_name".to_string()),
                new_site_url: "my_site_url".to_string(),
                new_note: Some("my_note".to_string())
            }),
            "ChangeWebsiteAccount\t1\tmy_account\tmy_password\tmy_site_name\tmy_site_url\tmy_note"
        );
        assert_eq!(
            depack_action(Action::DeleteWebsiteAccount { website_id: 1 }),
            "DeleteWebsiteAccount\t1"
        );
        assert_eq!(depack_action(Action::CheckDeadLink), "CheckDeadLink");
    }

    #[test]
    fn test_pack_ack() {
        let parts = vec!["Ack"];
        let ack = pack_ack(parts).unwrap();
        if let Ack::Ack = ack {
        } else {
            panic!("Ack error");
        }

        let parts = vec![
            "Info",
            "1\tmy_account\tmy_password\tmy_site_url\tmy_site_name\tmy_note\t1",
            "2\tmy_account\tmy_password\tmy_site_url\tmy_site_name\tmy_note\t1",
            "3\tmy_account\tmy_password\tmy_site_url\t\t\t0",
        ];
        let ack = pack_ack(parts).unwrap();
        if let Ack::Info { list } = ack {
            assert_eq!(list.len(), 3);
            assert_eq!(list[0].id, Some(1));
            assert_eq!(list[0].account, "my_account");
            assert_eq!(list[0].password, "my_password");
            assert_eq!(list[0].site_url, "my_site_url");
            assert_eq!(list[0].site_name, Some("my_site_name".to_string()));
            assert_eq!(list[0].note, Some("my_note".to_string()));
            assert_eq!(list[1].id, Some(2));
            assert_eq!(list[1].account, "my_account");
            assert_eq!(list[1].password, "my_password");
            assert_eq!(list[1].site_url, "my_site_url");
            assert_eq!(list[1].site_name, Some("my_site_name".to_string()));
            assert_eq!(list[1].note, Some("my_note".to_string()));
            assert_eq!(list[2].id, Some(3));
            assert_eq!(list[2].account, "my_account");
            assert_eq!(list[2].password, "my_password");
            assert_eq!(list[2].site_url, "my_site_url");
            assert_eq!(list[2].site_name, None);
            assert_eq!(list[2].note, None);
        } else {
            panic!("Info error");
        }

        let parts = vec!["DeadLink"];
        let ack = pack_ack(parts).unwrap();
        if let Ack::DeadLink { list: _ } = ack {
        } else {
            panic!("DeadLink error");
        }

        let parts = vec!["IdentityError"];
        let ack = pack_ack(parts).unwrap();
        if let Ack::IdentityError = ack {
        } else {
            panic!("IdentityError error");
        }

        let parts = vec!["DbError"];
        let ack = pack_ack(parts).unwrap();
        if let Ack::DbError = ack {
        } else {
            panic!("DbError error");
        }
    }

    // #[tokio::test]
    // async fn test_write_response() {
    //     let stream = TcpStream::connect("127.0.0.1:6123").await.unwrap();
    //     let action = Action::CheckIdentity {
    //         password: "my_password".to_string(),
    //     };
    //     write_response(&stream, action).await.unwrap();
    // }
}
