use crate::native_types::array::RArray;
use crate::native_types::bulk_string::RBulkString;
use crate::native_types::error::ErrorStruct;
use crate::native_types::redis_type::RedisType;

use crate::commands::database_mock::{DatabaseMock, TypeSaved};

pub struct LPop;

impl LPop {
    pub fn run(mut buffer: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        let key = String::from(buffer.remove(0));
        let count = parse_count(buffer)?;

        let popped: Vec<String> = Vec::new();
        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::List(list_of_values) => Ok(fill_list(popped, list_of_values, count)),
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided is not from strings"),
                )),
            }
        } else {
            Ok(RBulkString::encode("(nil)".to_string()))
        }
    }
}

fn parse_count(mut buffer: Vec<&str>) -> Result<usize, ErrorStruct> {
    if let Some(value) = buffer.pop() {
        if let Ok(counter) = value.parse::<usize>() {
            if counter > 0 {
                Ok(counter)
            } else {
                Err(ErrorStruct::new(
                    String::from("ERRUSIZE"),
                    String::from("provided counter is not a natural number"),
                ))
            }
        } else {
            Err(ErrorStruct::new(
                String::from("ERRUSIZE"),
                String::from("provided counter is not a natural number"),
            ))
        }
    } else {
        Ok(0)
    }
}

fn fill_list(mut popped: Vec<String>, list: &mut Vec<String>, counter: usize) -> String {
    if counter > 1 {
        for _ in 0..counter {
            popped.push(list.remove(0));
        }
        RArray::encode(popped)
    } else {
        RBulkString::encode(list.remove(0))
    }
}

#[cfg(test)]
pub mod test_lpush {

    use super::*;

    /*#[test]
    fn test01_lpop_values_on_an_existing_list() {
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
    }*/
}
