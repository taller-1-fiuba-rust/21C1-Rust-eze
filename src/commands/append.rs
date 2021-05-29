use crate::native_types::error::ErrorStruct;
use crate::native_types::integer::RInteger;
use crate::native_types::redis_type::RedisType;

use crate::commands::database_mock::{DatabaseMock, TypeSaved};

pub struct Append;

impl Append {
    pub fn run(mut buffer_vec: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        /*if let Some(strings) = database.get_mut_strings() {
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
            Err(ErrorStruct::new(
                "DATABASE".to_string(),
                "¿y ahora que?".to_string(),
            ))
        }*/

        let new_value = String::from(buffer_vec.pop().unwrap());
        let key = String::from(buffer_vec.pop().unwrap());
        let size: usize;
        if let Some(typesaved) = database.get_mut(&key) {

            match typesaved{

                TypeSaved::String(old_value) => {
                    old_value.push_str(&new_value);
                    size = old_value.len();
                    Ok(RInteger::encode(size as isize))

                }

                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided is not from strings"),
                ))

            }

        } else {
            size = new_value.len();
            database.insert(key, TypeSaved::String(new_value));
            Ok(RInteger::encode(size as isize))
        }

        

    }
}

#[cfg(test)]
pub mod test_append {

    use super::*;

    #[test]
    fn test01_append_to_an_existing_key() {
        let mut data = DatabaseMock::new();

        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer: Vec<&str> = vec!["key", "Appended"];
        let encoded = Append::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":13\r\n".to_string());
        assert_eq!(
            data.get("key"),
            Some(&TypeSaved::String("valueAppended".to_string()))
        );
    }

    #[test]
    fn test02_append_to_a_non_existing_key() {
        let mut data = DatabaseMock::new();
        let buffer: Vec<&str> = vec!["key", "newValue"];
        let encoded = Append::run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), ":8\r\n".to_string());
        assert_eq!(
            data.get("key"),
            Some(&TypeSaved::String("newValue".to_string()))
        );
    }
}
