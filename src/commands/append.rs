use std::collections::HashMap;

use crate::native_types::redis_type::RedisType;
use crate::native_types::integer::RInteger;
use crate::native_types::error::ErrorStruct;

pub struct Database {

    strings: HashMap<String, String>,

}

impl Database {

    pub fn new() -> Database {
        Database {
            strings: HashMap::new(),
        }
    }

    pub fn get_mut_strings(&mut self) -> Option<&mut HashMap<String, String>> {

        Some(&mut self.strings)

    }

}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
    

pub struct Append;

impl Append {

    pub fn run(mut buffer_vec: Vec<&str>, database: &mut Database) -> Result<String, ErrorStruct> {

        if let Some(strings) = database.get_mut_strings() {

            let new_value = String::from(buffer_vec.pop().unwrap());
            let key = String::from(buffer_vec.pop().unwrap());

            let size: usize;

            if let Some(old_value) = strings.get_mut(&key) {
                old_value.push_str(&new_value);
                size = old_value.len();
            } else {
                size = new_value.len();
                strings.insert(key, new_value);
            }

            Ok(RInteger::encode(size as isize))

        } else {

            Err(ErrorStruct::new("DATABASE".to_string(), "¿y ahora que?".to_string()))

        }

    }

}

#[cfg(test)]
pub mod test_append{

    use super::*;

    #[test]
    fn test01_append_to_an_existing_key(){

        let mut data = Database::new();

        {
            let strings = data.get_mut_strings().unwrap();

            strings.insert("key".to_string(), "value".to_string());
        }

        let buffer: Vec<&str> = vec!["key", "Appended"];
        let encoded = Append::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(data.get_mut_strings().unwrap().get("key"), Some(&"valueAppended".to_string()));

    }

    #[test]
    fn test02_append_to_a_non_existing_key(){

        let mut data = Database::new();
        let buffer: Vec<&str> = vec!["key", "newValue"];
        let encoded = Append::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":8\r\n".to_string());
        assert_eq!(data.get_mut_strings().unwrap().get("key"), Some(&"newValue".to_string()));
    }

}