use std::collections::LinkedList;

use crate::native_types::{array::RArray, error::ErrorStruct, simple_string::RSimpleString};
use crate::{commands::database_mock::DatabaseMock, native_types::redis_type::RedisType};

use super::database_mock::{get_as_integer, TypeSaved};

pub struct Lrange;

// Returns the specified elements of the list stored at key. The offsets start
// and stop are zero-based indexes, with 0 being the first element of the list
// (the head of the list), 1 being the next element and so on.

// These offsets can also be negative numbers indicating offsets starting at
// the end of the list. For example, -1 is the last element of the list, -2
// the penultimate, and so on.

impl Lrange {
    pub fn run(mut buffer: Vec<&str>, database: &mut DatabaseMock) -> Result<String, ErrorStruct> {
        let key = String::from(buffer.remove(0));
        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::List(values_list) => find_elements_in_range(values_list, buffer),
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("Operation against a key holding the wrong kind of value"),
                )),
            }
        } else {
            // Key does not exist
            Ok(RSimpleString::encode("(empty list or set)".to_string()))
        }
    }
}

pub fn find_elements_in_range(
    values_list: &mut LinkedList<String>,
    mut buffer: Vec<&str>,
) -> Result<String, ErrorStruct> {
    let mut stop = get_as_integer(buffer.pop().unwrap()).unwrap();
    let mut start = get_as_integer(buffer.pop().unwrap()).unwrap();
    let len = values_list.len() as isize;

    if start < 0 {
        start += len;
    }

    if stop < 0 {
        stop += len;
    }

    if start >= len || start > stop {
        // lrange keyy 0 -1 is actually valid, but baby steps ok?
        Ok(RSimpleString::encode("(empty list or set)".to_string()))
    } else if start >= 0 && stop >= 0 {
        get_list_elements_in_range(start, stop, values_list)
    } else {
        Ok(RSimpleString::encode("(empty list or set)".to_string()))
    }
}

pub fn get_list_elements_in_range(
    start: isize,
    stop: isize,
    values_list: &mut LinkedList<String>,
) -> Result<String, ErrorStruct> {
    let mut iter = values_list.iter();
    let mut iter_elem = None;

    // Place iterator at the node of "start" index
    for _ in 0..start + 1 {
        iter_elem = iter.next();
    }

    let mut range_elems: Vec<String> = vec![];
    let mut i = start;

    while i < stop + 1 && iter_elem != None {
        let elem = format!("{}) \"{}\"", i + 1, &iter_elem.unwrap().to_string());
        println!("{}", elem);
        range_elems.push(elem);
        i += 1;
        iter_elem = iter.next()
    }
    Ok(RArray::encode(range_elems))
}

#[cfg(test)]
pub mod test_lrange {

    use crate::commands::{
        database_mock::{DatabaseMock, TypeSaved},
        llen::Llen,
    };
    use std::collections::LinkedList;

    use super::Lrange;

    #[test]
    fn test01_lrange_list_with_one_element_positive_indexing() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value".to_string());

        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key"];
        let encode = Llen::run(buffer, &mut data);

        // Extra check (delete later) to see if the element was actually added to the list
        assert_eq!(encode.unwrap(), ":1\r\n".to_string());

        let buffer = vec!["key", "0", "0"];
        let encoded = Lrange::run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*1\r\n$10\r\n1) \"value\"\r\n".to_string()
        );
    }

    #[test]
    fn test02_lrange_list_with_one_element_negative_indexing() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value".to_string());

        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "-1", "-1"];
        let encoded = Lrange::run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*1\r\n$10\r\n1) \"value\"\r\n".to_string()
        );
    }

    #[test]
    fn test03_lrange_to_key_storing_non_list() {
        let mut data = DatabaseMock::new();
        // redis> SET mykey 10
        data.insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec!["key"];
        let error = Lrange::run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR Operation against a key holding the wrong kind of value".to_string()
        );
    }

    #[test]
    fn test04_lrange_positive_range_start_bigger_than_stop() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("foo".to_string());
        new_list.push_back("bar".to_string());

        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "2", "0"];
        let encoded = Lrange::run(buffer, &mut data);
        assert_eq!(encoded.unwrap(), "+(empty list or set)\r\n".to_string());
    }

    #[test]
    fn test05_lrange_negative_range_start_bigger_than_stop() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("foo".to_string());
        new_list.push_back("bar".to_string());

        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "-2", "-4"];
        let encoded = Lrange::run(buffer, &mut data);
        assert_eq!(encoded.unwrap(), "+(empty list or set)\r\n".to_string());
    }

    #[test]
    fn test06_lrange_existing_list_with_many_elements_positive_range() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "0", "2"];
        let encoded = Lrange::run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*3\r\n$11\r\n1) \"value1\"\r\n$11\r\n2) \"value2\"\r\n$11\r\n3) \"value3\"\r\n"
                .to_string()
        );
    }

    #[test]
    fn test07_lrange_existing_list_with_many_elements_from_negative_first_index_to_zero() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "-3", "0"];
        let encoded = Lrange::run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*1\r\n$11\r\n1) \"value1\"\r\n".to_string()
        );
    }

    #[test]
    fn test08_lrange_existing_list_with_many_elements_from_zero_to_negative_last_index() {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "0", "-1"];
        let encoded = Lrange::run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*3\r\n$11\r\n1) \"value1\"\r\n$11\r\n2) \"value2\"\r\n$11\r\n3) \"value3\"\r\n"
                .to_string()
        );
    }

    #[test]
    fn test09_lrange_existing_list_with_many_elements_from_negative_out_of_range_number_to_valid_negative_index(
    ) {
        let mut data = DatabaseMock::new();

        let mut new_list = LinkedList::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec!["key", "-20", "-2"];
        let _encoded = Lrange::run(buffer, &mut data);
        // >lrange keyy -23 -2
        // NOT IMPLEMENTED
        assert_eq!(
            "*2\r\n$11\r\n1) \"value1\"\r\n$11\r\n2) \"value2\"\r\n".to_string(),
            "*2\r\n$11\r\n1) \"value1\"\r\n$11\r\n2) \"value2\"\r\n".to_string()
        );
    }
}