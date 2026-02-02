use crate::protocol::RespType;
use crate::storage::Db;
use bytes::Bytes;

pub enum Command {
    Ping(Option<String>),
    Get(String),
    Set(String, Bytes),
    Del(String),
    Unknown(String),
}

impl Command {
    pub fn from_resp(resp: RespType) -> Result<Command, String> {
        // Redis commands are always come in arrays of bulkstrings

        let items = match resp {
            RespType::Array(items) => items,
            _ => return Err("Command must be an Array".to_string()),
        };

        if items.is_empty() {
            return Err("Empty command".to_string());
        }

        let command_name = match &items[0] {
            RespType::BulkString(bytes) => String::from_utf8(bytes.clone()).unwrap().to_uppercase(),
            _ => return Err("Command name must be a BulkString".to_string()),
        };

        match command_name.as_str() {
            "PING" => {
                if items.len() > 2 {
                    return Err("PING accepts at most 1 argument".to_string());
                }
                let msg = if items.len() == 2 {
                    match &items[1] {
                        RespType::BulkString(bytes) => Some(String::from_utf8(bytes.clone()).unwrap()),
                        _ => return Err("PING arguments must be a bulkString".to_string()),
                    } 
                } else {
                    None
                };
                Ok(Command::Ping(msg))
            }
            "GET" => {
                if items.len() != 2 {
                    return Err("GET requires exactly 1 argument".to_string());
                }
                let key = match &items[1] {
                    RespType::BulkString(bytes) => String::from_utf8(bytes.clone()).unwrap(),
                    _ => return Err("GET key must be a BulkString".to_string()),
                };
                Ok(Command::Get(key))
            }
            "SET" => {
                if items.len() != 3 {
                    return Err("SET require exactly 2 arguments".to_string());
                }
                let key = match &items[1] {
                    RespType::BulkString(bytes) => String::from_utf8(bytes.clone()).unwrap(),
                    _ => return Err("SET key must be a BulkString".to_string()),
                };
                let value = match &items[2] {
                    RespType::BulkString(bytes) => Bytes::from(bytes.clone()),
                    _ => return Err("SET value must be a BulkString".to_string()),
                };
                Ok(Command::Set(key, value))
            }
            "DEL" => {
                if items.len() != 2 {
                    return Err("DEL requires exactly 1 argument".to_string());
                }
                let key = match &items[1] {
                     RespType::BulkString(bytes) => String::from_utf8(bytes.clone()).unwrap(),
                     _ => return Err("DEL key must be a BulkString".to_string()),
                };
                Ok(Command::Del(key))
            }
            _ => Ok(Command::Unknown(command_name)),
        }


    }

    pub fn excute(self, db: &Db) -> RespType {
        match self {
            Command::Ping(msg) =>{
                match msg {
                    Some(s) => RespType::BulkString(s.into_bytes()),
                    None => RespType::SimpleString("PONG".to_string()),
                }

            }
            Command::Get(key) => {
                match db.get(&key) {
                    Some(value) => RespType::BulkString(value.to_vec()),
                    None => RespType::Null,
                }
            }
            Command::Set(key, val ) => {
                db.set(key, val);
                RespType::SimpleString("OK".to_string())
            }
            Command::Del(key) => {
                db.del(&key);
                RespType::Integer(1)
            }
            Command::Unknown(cmd) =>
                RespType::Error(format!("unknown command '{}'", cmd))
        }
    }
}