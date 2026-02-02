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

    pub fn execute(self, db: &Db) -> RespType {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;

    #[test]
    fn test_parse_get_command() {
        // Mock RESP array: ["GET", "key"]
        let input = RespType::Array(vec![
            RespType::BulkString(b"GET".to_vec()),
            RespType::BulkString(b"mykey".to_vec()),
        ]);

        let cmd = Command::from_resp(input).unwrap();
        
        match cmd {
            Command::Get(key) => assert_eq!(key, "mykey"),
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_set_and_get_execution() {
        let db = Db::new();

        // 1. Execute SET
        let set_cmd = Command::Set("foo".to_string(), Bytes::from("bar"));
        let res1 = set_cmd.execute(&db);
        
        match res1 {
            RespType::SimpleString(s) => assert_eq!(s, "OK"),
            _ => panic!("Expected OK response"),
        }

        // 2. Execute GET
        let get_cmd = Command::Get("foo".to_string());
        let res2 = get_cmd.execute(&db);

        match res2 {
            RespType::BulkString(data) => assert_eq!(data, b"bar"),
            _ => panic!("Expected BulkString('bar')"),
        }
    }

    #[test]
    fn test_parse_invalid_command() {
        // Mock SET command missing the value: ["SET", "key"]
        let input = RespType::Array(vec![
            RespType::BulkString(b"SET".to_vec()),
            RespType::BulkString(b"key".to_vec()),
        ]);

        let result = Command::from_resp(input);
        assert!(result.is_err()); // Should fail because SET needs 2 args
    }
    
    #[test]
    fn test_del_execution() {
        let db = Db::new();
        
        // 1. Set a key
        db.set("temp".to_string(), Bytes::from("val"));
        
        // 2. Delete it
        let del_cmd = Command::Del("temp".to_string());
        let res = del_cmd.execute(&db);
        
        match res {
            RespType::Integer(n) => assert_eq!(n, 1), // Should return 1 (deleted)
            _ => panic!("Expected Integer(1)"),
        }
        
        // 3. Verify it's gone
        assert!(db.get("temp").is_none());
    }
}