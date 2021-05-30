use crate::native_types::error::ErrorStruct;
use crate::native_types::integer::RInteger;
use crate::native_types::redis_type::RedisType;

use crate::commands::database_mock::{DatabaseMock, TypeSaved};

pub struct LPush;

impl LPush {
    pub fn run(mut buffer: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        let key = String::from(buffer.remove(0));
        let size;
        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::List(list_of_values) => {
                    fill_list(buffer, list_of_values);
                    size = list_of_values.len();
                    Ok(RInteger::encode(size as isize))
                }
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided is not from strings"),
                )),
            }
        } else {
            let mut new_list: Vec<String> = Vec::new();
            fill_list(buffer, &mut new_list);
            size = new_list.len();
            database.insert(key, TypeSaved::List(new_list));
            Ok(RInteger::encode(size as isize))
        }
    }
}

fn fill_list(mut buffer: Vec<&str>, list: &mut Vec<String>) {
    while !buffer.is_empty() {
        list.push(buffer.pop().unwrap().to_string());
    }
}

#[cfg(test)]
pub mod test_lpush {

    use super::*;

    #[test]
    fn test01_lpush_values_on_an_existing_list() {
        let mut data = DatabaseMock::new();
        let new_list: Vec<String> = vec![
            "this".to_string(),
            "is".to_string(),
            "a".to_string(),
            "list".to_string(),
        ];
        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "values", "new", "with"];
        let encode = LPush::run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":7\r\n".to_string());
        let fifth: &String;
        let sixth: &String;
        let seventh: &String;
        match data.get("key").unwrap() {
            TypeSaved::List(list) => {
                fifth = &list[4];
                sixth = &list[5];
                seventh = &list[6];
                assert_eq!(fifth, "with");
                assert_eq!(sixth, "new");
                assert_eq!(seventh, "values");
            }
            _ => {}
        }
    }

    #[test]
    fn test02_lpush_values_on_a_non_existing_list() {
        let mut data = DatabaseMock::new();
        let buffer: Vec<&str> = vec!["key", "this", "is", "a", "list"];
        let encode = LPush::run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":4\r\n".to_string());
        let first: &String;
        let second: &String;
        let third: &String;
        let fourth: &String;
        match data.get("key").unwrap() {
            TypeSaved::List(list) => {
                first = &list[0];
                second = &list[1];
                third = &list[2];
                fourth = &list[3];
                assert_eq!(first, "list");
                assert_eq!(second, "a");
                assert_eq!(third, "is");
                assert_eq!(fourth, "this");
            }
            _ => {}
        }
    }
}
