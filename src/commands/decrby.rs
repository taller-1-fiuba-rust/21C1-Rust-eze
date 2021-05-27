use super::database::{execute_value_modification, Database};

use crate::native_types::error::ErrorStruct;

pub struct Decrby;

/// Decrements the number stored at key by decrement. If the key does not exist, it is set
/// to 0 before performing the operation. An error is returned if the key contains a value
/// of the wrong type or contains a string that can not be represented as integer.
///
/// Operation is limited to 64 bit signed integers.

impl Decrby {
    pub fn run(buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {
        execute_value_modification(database, buffer_vec, decr)
    }
}

fn decr(minuend: isize, subtrahend: isize) -> isize {
    minuend - subtrahend
}

#[cfg(test)]
pub mod test_decrby {

    use super::*;

    #[test]
    fn test01_decrby_existing_key() {
        let mut data = Database::new();
        {
            // redis> SET mykey "10" ---> "OK"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "10".to_string());
        }
        // redis> DECRBY mykey 3 ---> (integer) 7
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":7\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"7".to_string())
        );
    }

    #[test]
    fn test02_decrby_existing_key_by_negative_integer() {
        let mut data = Database::new();
        {
            // redis> SET mykey "10"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "10".to_string());
        }
        // redis> DECRBY mykey -3
        let buffer: Vec<&str> = vec!["mykey", "-3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"13".to_string())
        );
    }

    #[test]
    fn test03_decrby_existing_key_with_negative_integer_value() {
        let mut data = Database::new();
        {
            // redis> SET mykey "-10"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "-10".to_string());
        }
        // redis> DECRBY mykey 3
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-13\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"-13".to_string())
        );
    }

    #[test]
    fn test04_decrby_existing_key_with_negative_integer_value_by_negative_integer() {
        let mut data = Database::new();
        {
            // redis> SET mykey "-10"
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "-10".to_string());
        }
        // redis> DECRBY mykey -3
        let buffer: Vec<&str> = vec!["mykey", "-3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-7\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"-7".to_string())
        );
    }

    #[test]
    fn test05_decrby_non_existing_key() {
        let mut data = Database::new();
        let buffer: Vec<&str> = vec!["mykey", "3"];
        let encoded = Decrby::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-3\r\n".to_string());
        assert_eq!(
            data.get_mut_strings().unwrap().get("mykey"),
            Some(&"-3".to_string())
        );
    }

    #[test]
    fn test06_decrby_existing_key_with_non_decrementable_value() {
        let mut data = Database::new();
        {
            // redis> SET mykey value
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "value".to_string());
        }
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
        let mut data = Database::new();
        {
            // redis> SET mykey 10
            let strings = data.get_mut_strings().unwrap();
            strings.insert("mykey".to_string(), "10".to_string());
        }
        // redis> DECRBY mykey a
        let buffer: Vec<&str> = vec!["mykey", "a"];
        let error = Decrby::run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }
}