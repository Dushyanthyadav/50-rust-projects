use bytes::{Buf, BytesMut};
use std::io::{Cursor, Read};

#[derive(Debug)]
pub enum RespType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<RespType>),
    Null,
}

impl RespType {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            RespType::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
            RespType::Error(msg) => format!("-{}\r\n", msg).into_bytes(),
            RespType::Integer(n) => format!(":{}\r\n", n).into_bytes(),
            RespType::BulkString(data) => {
                let mut out = format!("${}\r\n", data.len()).into_bytes();
                out.extend_from_slice(data);
                out.extend_from_slice(b"\r\n");
                out
            },
            RespType::Null => b"$-1\r\n".to_vec(),
            RespType::Array(_) => b"-ERR array serialization not supported\r\n".to_vec(),
        }
    }
}

#[derive(Debug)]
pub enum RespError {
    InvalidProtocol,
    Utf8Error,
    IntError,
}

pub fn decode(buff: &mut BytesMut) -> Result<Option<RespType>, RespError> {
    // exit if buff is empty happens if the stream coming in(tcp is a stream so there is no way for use to know end of message)
    if buff.is_empty() {
        return Ok(None);
    }

    // warp the buff in cursor for position and reading
    let mut cursor = Cursor::new(&buff[..]);

    // parse_next is the real parseing function 
    // only the decode function can advance the buffer clearing the old frame
    match parse_next(&mut cursor) {
        Ok(Some(value)) => {
            let position = cursor.position() as usize;
            buff.advance(position);
            Ok(Some(value))
        }
        Ok(None) => Ok(None),

        Err(e) => Err(e),
    }
}

fn parse_next(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespType>, RespError> {
    let mut prefix_byte = [0u8; 1];
    // This check is crucial because we want to know if buffer is empty or not in get_array
    if cursor.read_exact(&mut prefix_byte).is_err() {
        return Ok(None);
    }

    match prefix_byte[0] {
        b'+' => get_simple_string(cursor),
        b':' => get_decimal(cursor),
        b'$' => get_bulk_string(cursor),
        b'*' => get_array(cursor),
        _ => Err(RespError::InvalidProtocol),
    }
}

fn get_simple_string(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespType>, RespError> {
    let start = cursor.position() as usize;
    let end = cursor.get_ref().len();

    let window = &cursor.get_ref()[start..end];
    // Here we are creating a window of size 2 it is matching for 2 pattern the window can overlap
    match window.windows(2).position(|w| w == b"\r\n") {
        Some(index) => {
            let data_end = start + index;
            let string_bytes = &cursor.get_ref()[start..data_end];

            let s = String::from_utf8(string_bytes.to_vec()).map_err(|_| RespError::Utf8Error)?;

            cursor.set_position((data_end + 2) as u64);
            Ok(Some(RespType::SimpleString(s)))
        }

        None => Ok(None),
    }
}

fn get_decimal(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespType>, RespError> {
    let start = cursor.position() as usize;
    let end = cursor.get_ref().len();

    let window = &cursor.get_ref()[start..end];

    match window.windows(2).position(|w| w == b"\r\n") {
        Some(index) => {
            let data_end = start + index;
            let integer_bytes = &cursor.get_ref()[start..data_end];

            let num_str =
                String::from_utf8(integer_bytes.to_vec()).map_err(|_| RespError::Utf8Error)?;

            let num = num_str.parse::<i64>().map_err(|_| RespError::IntError)?;

            cursor.set_position((data_end + 2) as u64);

            Ok(Some(RespType::Integer(num)))
        }

        None => Ok(None),
    }
}

fn get_bulk_string(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespType>, RespError> {
    let start = cursor.position() as usize;
    let end = cursor.get_ref().len();

    // There is 2 reads in this. one for number of bytes and other for reading the data;
    let index = match cursor.get_ref()[start..end]
        .windows(2)
        .position(|w| w == b"\r\n")
    {
        Some(i) => i,
        None => return Ok(None),
    };

    let header_end = start + index;
    let len_str_bytes = &cursor.get_ref()[start..header_end];
    let len_str = String::from_utf8(len_str_bytes.to_vec()).map_err(|_| RespError::Utf8Error)?;

    let len = len_str.parse::<i64>().map_err(|_| RespError::IntError)?;

    if len == -1 {
        cursor.set_position((header_end + 2) as u64);
        return Ok(Some(RespType::Null));
    }

    let data_len = len as usize;
    let total_frame_len = header_end + 2 + data_len + 2;

    if end < total_frame_len {
        return Ok(None);
    }

    let data_start = header_end + 2;
    let data_end = data_start + data_len;

    let data = cursor.get_ref()[data_start..data_end].to_vec();

    cursor.set_position(total_frame_len as u64);

    Ok(Some(RespType::BulkString(data)))
}

fn get_array(cursor: &mut Cursor<&[u8]>) -> Result<Option<RespType>, RespError> {
    let start = cursor.position() as usize;
    let end = cursor.get_ref().len();
    // This function as well as 2 reads one for number of elements and the elements itself
    let index = match cursor.get_ref()[start..end]
        .windows(2)
        .position(|w| w == b"\r\n")
    {
        Some(i) => i,
        None => return Ok(None),
    };

    let header_end = start + index;
    let count_str = String::from_utf8(cursor.get_ref()[start..header_end].to_vec())
        .map_err(|_| RespError::Utf8Error)?;

    let count = count_str.parse::<i64>().map_err(|_| RespError::IntError)?;

    cursor.set_position((header_end + 2) as u64);

    if count == -1 {
        return Ok(Some(RespType::Null));
    }

    let mut items = Vec::with_capacity(count as usize);
    for _ in 0..count {
        match parse_next(cursor)? {
            Some(value) => items.push(value),
            None => return Ok(None),
        }
    }

    Ok(Some(RespType::Array(items)))
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_simple_string_success() {
        let mut buffer = BytesMut::from("+OK\r\n");

        let result = decode(&mut buffer).unwrap();

        match result {
            Some(RespType::SimpleString(s)) => assert_eq!(s, "OK"),
            _ => panic!("Expected SimpleString('OK'), got {:?}", result),
        }

       // verifying if buffer is 0 or not
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_simple_string_incomplete() {
        let mut buffer = BytesMut::from("+Hel");

        let result = decode(&mut buffer).unwrap();

        assert!(result.is_none());

        // Buffer should not change
        assert_eq!(buffer.len(), 4);
    }

    #[test]
    fn test_decimal_success() {
        let mut buffer = BytesMut::from(":1000\r\n");
        let result = decode(&mut buffer).unwrap();

        match result {
            Some(RespType::Integer(val)) => assert_eq!(val, 1000),
            _ => panic!("Expected Integer(1000), got {:?}", result),
        }
        assert_eq!(buffer.len(), 0); // Buffer should be consumed
    }

    #[test]
    fn test_decimal_negative() {
        let mut buffer = BytesMut::from(":-42\r\n");
        let result = decode(&mut buffer).unwrap();

        match result {
            Some(RespType::Integer(val)) => assert_eq!(val, -42),
            _ => panic!("Expected Integer(-42), got {:?}", result),
        }
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_decimal_incomplete() {
        let mut buffer = BytesMut::from(":100"); // Missing \r\n
        let result = decode(&mut buffer).unwrap();

        assert!(result.is_none());
        assert_eq!(buffer.len(), 4); // Nothing should be consumed
    }

    #[test]
    fn test_decimal_invalid() {
        // Not a interger
        let mut buffer = BytesMut::from(":ABC\r\n");
        let result = decode(&mut buffer);

        match result {
            Err(RespError::IntError) => (), // Pass
            _ => panic!("Expected IntError, got {:?}", result),
        }
        assert_eq!(buffer.len(), 6);
    }

    #[test]
    fn test_bulk_string_success() {
        // Standard Bulk String: $5\r\nhello\r\n
        let mut buffer = BytesMut::from("$5\r\nhello\r\n");
        let result = decode(&mut buffer).unwrap();

        match result {
            Some(RespType::BulkString(data)) => assert_eq!(data, b"hello"),
            _ => panic!("Expected BulkString('hello'), got {:?}", result),
        }
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_bulk_string_null() {
        // Redis Null Bulk String: $-1\r\n
        let mut buffer = BytesMut::from("$-1\r\n");
        let result = decode(&mut buffer).unwrap();

        match result {
            Some(RespType::Null) => (), // Pass
            _ => panic!("Expected Null, got {:?}", result),
        }
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_bulk_string_incomplete() {
        // Buffer has the header but is missing the actual data
        let mut buffer = BytesMut::from("$5\r\nhel");
        let result = decode(&mut buffer).unwrap();

        assert!(result.is_none());
        assert_eq!(buffer.len(), 7); // Buffer remains untouched
    }

    #[test]
    fn test_array_simple() {
        // Array of two bulk strings: *2\r\n$4\r\necho\r\n$5\r\nhello\r\n
        let mut buffer = BytesMut::from("*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");
        let result = decode(&mut buffer).unwrap();

        match result {
            Some(RespType::Array(items)) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    RespType::BulkString(b) => assert_eq!(b, b"echo"),
                    _ => panic!("Expected BulkString"),
                }
            }
            _ => panic!("Expected Array, got {:?}", result),
        }
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_array_nested() {
        // Nested Array: *1\r\n*1\r\n:5\r\n
        // (An array containing an array containing the integer 5)
        let mut buffer = BytesMut::from("*1\r\n*1\r\n:5\r\n");
        let result = decode(&mut buffer).unwrap();

        if let Some(RespType::Array(ref outer)) = result {
            if let RespType::Array(inner) = &outer[0] {
                if let RespType::Integer(val) = inner[0] {
                    assert_eq!(val, 5);
                    return;
                }
            }
        }
        panic!("Nested array parsing failed! Got: {:?}", result);
    }

    #[test]
    fn test_array_incomplete() {
        // Array header says 2 elements, but we only provide one
        let mut buffer = BytesMut::from("*2\r\n:100\r\n");
        let result = decode(&mut buffer).unwrap();

        assert!(result.is_none());
        assert_eq!(buffer.len(), 10); // Entire buffer should remain untouched
    }

    #[test]
    fn test_overall_protocol_pipeline() {
        let mut buffer = BytesMut::from("+OK\r\n:100\r\n$5\r\nhello\r\n");

        let res1 = decode(&mut buffer).unwrap();
        match res1 {
            Some(RespType::SimpleString(s)) => assert_eq!(s, "OK"),
            _ => panic!("Expected OK, got {:?}", res1),
        }
        
        assert_eq!(buffer.len(), 17); // +OK\r\n removed 

        
        let res2 = decode(&mut buffer).unwrap();
        match res2 {
            Some(RespType::Integer(i)) => assert_eq!(i, 100),
            _ => panic!("Expected 100, got {:?}", res2),
        }
        assert_eq!(buffer.len(), 11); // Removed :100\r\n (6 bytes)

        
        let res3 = decode(&mut buffer).unwrap();
        match res3 {
            Some(RespType::BulkString(b)) => assert_eq!(b, b"hello"),
            _ => panic!("Expected hello, got {:?}", res3),
        }
        assert_eq!(buffer.len(), 0); // Everything gone!
    }

    #[test]
    fn test_empty_buffer() {
        let mut buffer = BytesMut::new();
        let result = decode(&mut buffer).unwrap();
        assert!(result.is_none());
    }
#[test]
    fn test_serialize_simple_string() {
        let resp = RespType::SimpleString("OK".to_string());
        assert_eq!(resp.serialize(), b"+OK\r\n");
    }

    #[test]
    fn test_serialize_bulk_string() {
        let resp = RespType::BulkString(b"hello".to_vec());
        assert_eq!(resp.serialize(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_serialize_integer() {
        let resp = RespType::Integer(42);
        assert_eq!(resp.serialize(), b":42\r\n");
    }

    #[test]
    fn test_serialize_null() {
        let resp = RespType::Null;
        assert_eq!(resp.serialize(), b"$-1\r\n");
    }

    #[test]
    fn test_serialize_error() {
        let resp = RespType::Error("Error message".to_string());
        assert_eq!(resp.serialize(), b"-Error message\r\n");
    }
}
