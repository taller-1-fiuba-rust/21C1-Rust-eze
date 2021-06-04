use crate::{
    commands::{check_empty, Runnable},
    database::{Database, TypeSaved},
    err_wrongtype,
    messages::redis_messages,
    native_types::{ErrorStruct, RInteger, RedisType},
};
use std::collections::HashSet;

pub struct Sadd;

impl Runnable for Sadd {
    fn run(
        &self,
        mut buffer_vec: Vec<&str>,
        database: &mut Database,
    ) -> Result<String, ErrorStruct> {
        check_error_cases(&mut buffer_vec)?;

        let key = buffer_vec[0];

        match database.get_mut(key) {
            Some(item) => match item {
                TypeSaved::Set(item) => {
                    let count_insert = insert_in_set(&buffer_vec, item);
                    Ok(RInteger::encode(count_insert as isize))
                }
                _ => {
                    err_wrongtype!()
                }
            },
            None => {
                let mut set: HashSet<String> = HashSet::new();
                let count_insert = insert_in_set(&buffer_vec, &mut set);
                database.insert(key.to_string(), TypeSaved::Set(set));
                Ok(RInteger::encode(count_insert as isize))
            }
        }
    }
}
// Insert the "members" into the received set, according to what is indicated by the vector buffer (for example: "sadd key member1 member2 ..")
// Returns the number of insertions new in the set (repeated ones is ignored)
fn insert_in_set(buffer_vec: &[&str], item: &mut HashSet<String>) -> usize {
    buffer_vec
        .iter()
        .skip(1)
        .map(|member| item.insert(member.to_string()))
        .filter(|x| *x)
        .count()
}

fn check_error_cases(buffer_vec: &mut Vec<&str>) -> Result<(), ErrorStruct> {
    check_empty(&buffer_vec, "sadd")?;

    if buffer_vec.len() < 2 {
        let error_message = redis_messages::arguments_invalid_to("sadd");
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test_sadd_function {

    use std::collections::LinkedList;

    use super::*;

    #[test]
    fn test01_sadd_insert_and_return_amount_insertions() {
        let buffer_vec_mock = vec!["key", "member1", "member2"];
        let mut database_mock = Database::new();

        let result_received = Sadd.run(buffer_vec_mock, &mut database_mock);
        let amount_received = result_received.unwrap();

        let expected = RInteger::encode(2);
        assert_eq!(expected, amount_received);
    }

    #[test]
    fn test02_sadd_does_not_insert_repeated_elements() {
        let buffer_vec_mock = vec![
            "key", "member2", "member1", "member1", "member3", "member2", "member1", "member1",
            "member3",
        ];
        let mut database_mock = Database::new();

        let result_received = Sadd.run(buffer_vec_mock, &mut database_mock);
        let amount_received = result_received.unwrap();

        let expected = RInteger::encode(3);
        assert_eq!(expected, amount_received);
    }

    #[test]
    fn test03_sadd_does_not_insert_elements_over_an_existing_key_string() {
        let mut database_mock = Database::new();
        database_mock.insert("key".to_string(), TypeSaved::String("value".to_string()));
        let buffer_vec_mock = vec![
            "key", "member2", "member1", "member1", "member3", "member2", "member1", "member1",
            "member3",
        ];

        let result_received = Sadd.run(buffer_vec_mock, &mut database_mock);

        assert!(result_received.is_err())
    }

    #[test]
    fn test04_sadd_does_not_insert_elements_over_an_existing_key_list() {
        let mut database_mock = Database::new();
        let mut new_list = LinkedList::new();
        new_list.push_back("valueOfList".to_string());
        database_mock.insert("key".to_string(), TypeSaved::List(new_list));

        let buffer_vec_mock = vec![
            "key", "member2", "member1", "member1", "member3", "member2", "member1", "member1",
            "member3",
        ];

        let result_received = Sadd.run(buffer_vec_mock, &mut database_mock);

        assert!(result_received.is_err())
    }
}
