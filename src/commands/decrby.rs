use super::database_mock::{execute_value_modification, DatabaseMock};
use crate::native_types::error::ErrorStruct;

pub struct Decrby;

/// Decrements the number stored at key by decrement. If the key does not exist, it is set
/// to 0 before performing the operation. An error is returned if the key contains a value
/// of the wrong type or contains a string that can not be represented as integer.
///
/// Operation is limited to 64 bit signed integers.

impl Decrby {
    pub fn run(buffer_vec: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        execute_value_modification(database, buffer_vec, decr)
    }
}

fn decr(minuend: isize, subtrahend: isize) -> isize {
    minuend - subtrahend
}

#[cfg(test)]
pub mod test_decrby {

    use crate::commands::database_mock::TypeSaved;

    use super::*;

    #[test]
    fn test01_decrby_existing_key() {
        let mut data = DatabaseMock::new();
        // redis> SET mykey 10
        data.insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> DECRBY mykey 3 ---> (integer) 7
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":7\r\n".to_string());
        assert_eq!(data.get("mykey"), Some(&TypeSaved::String("7".to_string())));
    }

    #[test]
    fn test02_decrby_existing_key_by_negative_integer() {
        let mut data = DatabaseMock::new();
        // redis> SET mykey 10
        data.insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> DECRBY mykey -3
        let buffer: Vec<&str> = vec!["mykey", "-3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(
            data.get("mykey"),
            Some(&TypeSaved::String("13".to_string()))
        );
    }

    #[test]
    fn test03_decrby_existing_key_with_negative_integer_value() {
        let mut data = DatabaseMock::new();
        // redis> SET mykey -10
        data.insert("mykey".to_string(), TypeSaved::String("-10".to_string()));
        // redis> DECRBY mykey 3
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-13\r\n".to_string());
        assert_eq!(
            data.get("mykey"),
            Some(&TypeSaved::String("-13".to_string()))
        );
    }

    #[test]
    fn test04_decrby_existing_key_with_negative_integer_value_by_negative_integer() {
        let mut data = DatabaseMock::new();
        // redis> SET mykey -10
        data.insert("mykey".to_string(), TypeSaved::String("-10".to_string()));
        // redis> DECRBY mykey -3
        let buffer: Vec<&str> = vec!["mykey", "-3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-7\r\n".to_string());
        assert_eq!(
            data.get("mykey"),
            Some(&TypeSaved::String("-7".to_string()))
        );
    }

    #[test]
    fn test05_decrby_non_existing_key() {
        let mut data = DatabaseMock::new();
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-3\r\n".to_string());
        assert_eq!(
            data.get("mykey"),
            Some(&TypeSaved::String("-3".to_string()))
        );
    }

    #[test]
    fn test06_decrby_existing_key_with_non_decrementable_value() {
        let mut data = DatabaseMock::new();
        // redis> SET mykey value
        data.insert("mykey".to_string(), TypeSaved::String("value".to_string()));
        // redis> DECRBY mykey 1
        let buffer: Vec<&str> = vec!["mykey", "value"];
        let error = Decrby::run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }

    #[test]
    fn test07_decrby_existing_key_by_non_integer() {
        let mut data = DatabaseMock::new();
        // redis> SET mykey 10
        data.insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> DECRBY mykey a
        let buffer: Vec<&str> = vec!["mykey", "a"];
        let error = Decrby::run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }
}
