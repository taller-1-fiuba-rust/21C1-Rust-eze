use crate::commands::lists::{check_empty, check_not_empty};
use crate::commands::{get_as_integer, Runnable};
use crate::database::Database;
use crate::database::TypeSaved;
use crate::messages::redis_messages;
use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::RedisType;
use crate::native_types::{array::RArray, error::ErrorStruct, simple_string::RSimpleString};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
pub struct Lrange;

impl Runnable<Arc<Mutex<Database>>> for Lrange {
    /// Returns the specified elements of the list stored at key. The offsets start and
    /// stop are zero-based indexes, with 0 being the first element of the list (the head
    /// of the list), 1 being the next element and so on.
    /// These offsets can also be negative numbers indicating offsets starting at the end
    /// of the list. For example, -1 is the last element of the list, -2 the penultimate,
    /// and so on.
    /// Out of range indexes will not produce an error. If start is larger than the end of
    /// the list, an empty list is returned. If stop is larger than the actual end of the
    /// list, Redis will treat it like the last element of the list.
    /// Time complexity: O(S+N) where S is the distance of start offset from HEAD for small
    /// lists, from nearest end (HEAD or TAIL) for large lists; and N is the number of
    /// elements in the specified range.
    ///
    /// # Return value
    /// [String] _encoded_ in [RArray]: list of elements in the specified range.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a list.
    /// * **Key** does not exist.
    /// * Buffer [Vec]<[String]> is received empty, or received with an amount of elements different than 3.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
    fn run(
        &self,
        mut buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        check_empty(&buffer, "lrange")?;
        let key = buffer.remove(0);
        if let Some(typesaved) = database.get_mut(&key) {
            match typesaved {
                TypeSaved::List(values_list) => find_elements_in_range(values_list, buffer),
                _ => Err(ErrorStruct::new(
                    String::from("WRONGTYPE"),
                    String::from("Operation against a key holding the wrong kind of value"),
                )),
            }
        } else {
            // Key does not exist
            Ok(RSimpleString::encode("(empty list or set)".to_string()))
        }
    }
}

// Obtains list stop and start indexes checking buffer has only 2 elements. If
// indexes are not valid, returns "(empty list or set)", any other case, returns
// a decoded RArray containing all elements at interval [start, stop].
pub fn find_elements_in_range(
    values_list: &mut VecDeque<String>,
    mut buffer: Vec<String>,
) -> Result<String, ErrorStruct> {
    check_empty(&buffer, "lrange")?;
    let mut stop = get_as_integer(&buffer.pop().unwrap()).unwrap();
    check_empty(&buffer, "lrange")?;
    let mut start = get_as_integer(&buffer.pop().unwrap()).unwrap();
    check_not_empty(&buffer)?;
    let len = values_list.len() as isize;
    if start < 0 {
        start += len; // start = 2
    }
    if stop < 0 {
        stop += len; // stop = 2
    }
    if start >= len || start > stop || stop < -len {
        Ok(RSimpleString::encode("(empty list or set)".to_string()))
    } else {
        if start < 0 {
            start = 0;
        }
        if stop >= len {
            stop = len - 1; // check if -1 is needed
        }
        get_list_elements_in_range(start, stop, values_list)
    }
}

// Iterates the VecDeque pushing all elements in interval [start, stop]
// to a Vec<String> and returns it encoded as RArray.
pub fn get_list_elements_in_range(
    start: isize,
    stop: isize,
    values_list: &mut VecDeque<String>,
) -> Result<String, ErrorStruct> {
    let mut iter = values_list.iter();
    let mut iter_elem = None;

    // Place iterator at the node of "start" index
    for _ in 0..start + 1 {
        iter_elem = iter.next();
    }

    let mut range_elems: Vec<String> = vec![];
    let mut i = start;
    let mut j = 1;
    while i < stop + 1 && iter_elem != None {
        let elem = format!("{}) \"{}\"", j, &iter_elem.unwrap().to_string());
        println!("{}", elem);
        range_elems.push(elem);
        i += 1;
        j += 1;
        iter_elem = iter.next()
    }
    Ok(RArray::encode(range_elems))
}

#[cfg(test)]
pub mod test_lrange {
    use crate::commands::create_notifier;

    use crate::{
        commands::{lists::llen::Llen, Runnable},
        vec_strings,
    };

    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test_01_lrange_list_with_one_element_positive_indexing() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key"];
        let encode = Llen.run(buffer, &mut data);

        // Extra check (delete later) to see if the element was actually added to the list
        assert_eq!(encode.unwrap(), ":1\r\n".to_string());

        let buffer = vec_strings!["key", "0", "0"];
        let encoded = Lrange.run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*1\r\n$10\r\n1) \"value\"\r\n".to_string()
        );
    }

    #[test]
    fn test_02_lrange_list_with_one_element_negative_indexing() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-1", "-1"];
        let encoded = Lrange.run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*1\r\n$10\r\n1) \"value\"\r\n".to_string()
        );
    }

    #[test]
    fn test_03_lrange_to_key_storing_non_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        // redis> SET mykey 10
        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec_strings!["key"];
        let error = Lrange.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
        );
    }

    #[test]
    fn test_04_lrange_positive_range_start_bigger_than_stop() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("foo".to_string());
        new_list.push_back("bar".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "2", "0"];
        let encoded = Lrange.run(buffer, &mut data);
        assert_eq!(encoded.unwrap(), "+(empty list or set)\r\n".to_string());
    }

    #[test]
    fn test_05_lrange_negative_range_start_bigger_than_stop() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("foo".to_string());
        new_list.push_back("bar".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-2", "-4"];
        let encoded = Lrange.run(buffer, &mut data);
        assert_eq!(encoded.unwrap(), "+(empty list or set)\r\n".to_string());
    }

    #[test]
    fn test_06_lrange_list_with_many_elements_positive_range() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "0", "2"];
        let encoded = Lrange.run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*3\r\n$11\r\n1) \"value1\"\r\n$11\r\n2) \"value2\"\r\n$11\r\n3) \"value3\"\r\n"
                .to_string()
        );
    }

    #[test]
    fn test_07_lrange_list_with_many_elements_from_negative_first_index_to_zero() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-3", "0"];
        let encoded = Lrange.run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*1\r\n$11\r\n1) \"value1\"\r\n".to_string()
        );
    }

    #[test]
    fn test_08_lrange_list_with_many_elements_from_zero_to_negative_last_index() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "0", "-1"];
        let encoded = Lrange.run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*3\r\n$11\r\n1) \"value1\"\r\n$11\r\n2) \"value2\"\r\n$11\r\n3) \"value3\"\r\n"
                .to_string()
        );
    }

    #[test]
    fn test_09_lrange_list_with_many_elements_from_negative_out_of_range_number_to_valid_negative_index(
    ) {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-20", "-2"];
        let encoded = Lrange.run(buffer, &mut data);
        // >lrange keyy -23 -2
        assert_eq!(
            encoded.unwrap(),
            "*2\r\n$11\r\n1) \"value1\"\r\n$11\r\n2) \"value2\"\r\n".to_string()
        );
    }

    #[test]
    fn test_10_lrange_list_with_many_elements_from_negative_out_of_range_number_to_invalid_negative_index(
    ) {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-20", "-10"];
        // >lrange keyy -20 -10
        let encoded = Lrange.run(buffer, &mut data);
        assert_eq!(encoded.unwrap(), "+(empty list or set)\r\n".to_string());
    }

    #[test]
    fn test_11_lrange_list_with_many_elements_from_negative_out_of_range_number_to_number_bigger_than_len(
    ) {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-20", "20"];
        let encoded = Lrange.run(buffer, &mut data);
        // >lrange keyy -20 20
        assert_eq!(
            encoded.unwrap(),
            "*3\r\n$11\r\n1) \"value1\"\r\n$11\r\n2) \"value2\"\r\n$11\r\n3) \"value3\"\r\n"
                .to_string()
        );
    }

    #[test]
    fn test_12_lrange_list_with_many_elements_from_negative_out_of_range_number_to_list_bottom() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-20", "-1"];
        let encoded = Lrange.run(buffer, &mut data);
        // >lrange keyy -20 -1
        assert_eq!(
            encoded.unwrap(),
            "*3\r\n$11\r\n1) \"value1\"\r\n$11\r\n2) \"value2\"\r\n$11\r\n3) \"value3\"\r\n"
                .to_string()
        );
    }

    #[test]
    fn test_13_lrange_list_many_element_negative_indexing() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-1", "-1"];
        let encoded = Lrange.run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*1\r\n$11\r\n1) \"value3\"\r\n".to_string()
        );
    }

    #[test]
    fn test_14_lrange_list_many_element_from_negative_index_to_zero() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        let mut new_list = VecDeque::new();
        new_list.push_back("value1".to_string());
        new_list.push_back("value2".to_string());
        new_list.push_back("value3".to_string());

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "-3", "0"];
        let encoded = Lrange.run(buffer, &mut data);
        assert_eq!(
            encoded.unwrap(),
            "*1\r\n$11\r\n1) \"value1\"\r\n".to_string()
        );
    }
}
