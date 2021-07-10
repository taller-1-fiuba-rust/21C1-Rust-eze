use crate::{commands::Runnable, database::Database, native_types::error::ErrorStruct};

use super::execute_value_modification;

pub struct Incrby;

/// Increments the number stored at key by increment. If the key does not exist, it is set
/// to 0 before performing the operation. An error is returned if the key contains a value
/// of the wrong type or contains a string that can not be represented as integer.
///
/// This operation is limited to 64 bit signed integers.

impl Runnable<Database> for Incrby {
    fn run(&self, buffer: Vec<String>, database: &mut Database) -> Result<String, ErrorStruct> {
        execute_value_modification(database, buffer, incr)
    }
}

fn incr(addend1: isize, addend2: isize) -> isize {
    addend1 + addend2
}

#[cfg(test)]
pub mod test_incrby {

    use crate::{
        database::{Database, TypeSaved},
        vec_strings,
    };

    use super::*;

    #[test]
    fn test01_incrby_existing_key() {
        let mut data = Database::new();
        // redis> SET mykey 10
        data.insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> INCRBY mykey 3 ---> (integer) 13
        let buffer = vec_strings!["mykey", "3"];
        let encoded = Incrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(
            data.get("mykey"),
            Some(&TypeSaved::String("13".to_string()))
        );
    }

    #[test]
    fn test02_incrby_existing_key_by_negative_integer() {
        let mut data = Database::new();
        // redis> SET mykey 10
        data.insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> INCRBY mykey -3
        let buffer = vec_strings!["mykey", "-3"];
        let encoded = Incrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":7\r\n".to_string());
        assert_eq!(data.get("mykey"), Some(&TypeSaved::String("7".to_string())));
    }

    #[test]
    fn test03_incrby_existing_key_with_negative_integer_value() {
        let mut data = Database::new();
        // redis> SET mykey -10
        data.insert("mykey".to_string(), TypeSaved::String("-10".to_string()));
        // redis> INCRBY mykey 3
        let buffer = vec_strings!["mykey", "3"];
        let encoded = Incrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-7\r\n".to_string());
        assert_eq!(
            data.get("mykey"),
            Some(&TypeSaved::String("-7".to_string()))
        );
    }

    #[test]
    fn test04_incrby_existing_key_with_negative_integer_value_by_negative_integer() {
        let mut data = Database::new();
        // redis> SET mykey -10
        data.insert("mykey".to_string(), TypeSaved::String("-10".to_string()));
        // redis> INCRBY mykey -3
        let buffer = vec_strings!["mykey", "-3"];
        let encoded = Incrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":-13\r\n".to_string());
        assert_eq!(
            data.get("mykey"),
            Some(&TypeSaved::String("-13".to_string()))
        );
    }

    #[test]
    fn test05_incrby_non_existing_key() {
        let mut data = Database::new();
        let buffer = vec_strings!["mykey", "3"];
        let encoded = Incrby.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":3\r\n".to_string());
        assert_eq!(data.get("mykey"), Some(&TypeSaved::String("3".to_string())));
    }

    #[test]
    fn test06_incrby_existing_key_with_non_decrementable_value() {
        let mut data = Database::new();
        // redis> SET mykey value
        data.insert("mykey".to_string(), TypeSaved::String("value".to_string()));
        // redis> INCRBY mykey 1
        let buffer = vec_strings!["mykey", "value"];
        let error = Incrby.run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }

    #[test]
    fn test07_decrby_existing_key_by_non_integer() {
        let mut data = Database::new();
        // redis> SET mykey 10
        data.insert("mykey".to_string(), TypeSaved::String("10".to_string()));
        // redis> INCRBY mykey a
        let buffer = vec_strings!["mykey", "a"];
        let error = Incrby.run(buffer, &mut data);

        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR value is not an integer or out of range".to_string()
        );
    }
}
