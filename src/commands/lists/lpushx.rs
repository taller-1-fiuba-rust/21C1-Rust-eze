use super::fill_list_from_top;
use super::pushx_at;
use crate::commands::Runnable;
use crate::database::Database;
use crate::messages::redis_messages;
use crate::native_types::error::ErrorStruct;
use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};

pub struct LPushx;

impl Runnable<Arc<Mutex<Database>>> for LPushx {
    /// Inserts specified values at the head of the list stored at key, only if key already
    /// exists and holds a list. In contrary to LPUSH, no operation will be performed when
    /// key does not yet exist.
    ///
    /// # Return value
    /// [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): the length of the list after the push operation.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The value stored at **key** is not a list.
    /// * Buffer [Vec]<[String]> is received empty, or received with more than 2 elements.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.
    fn run(
        &self,
        buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        pushx_at(buffer, &mut database, fill_list_from_top)
    }
}

#[cfg(test)]
pub mod test_lpushx {

    use crate::commands::create_notifier;
    use std::collections::VecDeque;

    use crate::{database::TypeSaved, vec_strings};

    use super::*;

    #[test]
    fn test_01_lpushx_values_on_an_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let c_data = Arc::clone(&data);

        let mut new_list = VecDeque::new();
        new_list.push_back("with".to_string());
        new_list.push_back("new".to_string());
        new_list.push_back("values".to_string());
        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::List(new_list));

        let buffer = vec_strings!["key", "list", "a", "is", "this"];
        let encode = LPushx.run(buffer, &mut data);
        assert_eq!(encode.unwrap(), ":7\r\n".to_string());
        let mut c_db = c_data.lock().unwrap();

        match c_db.get_mut("key").unwrap() {
            TypeSaved::List(list) => {
                assert_eq!(list.pop_front().unwrap(), "this");
                assert_eq!(list.pop_front().unwrap(), "is");
                assert_eq!(list.pop_front().unwrap(), "a");
                assert_eq!(list.pop_front().unwrap(), "list");
                assert_eq!(list.pop_front().unwrap(), "with");
                assert_eq!(list.pop_front().unwrap(), "new");
                assert_eq!(list.pop_front().unwrap(), "values");
            }
            _ => {}
        }
    }

    #[test]
    fn test_02_lpushx_values_on_a_non_existing_list() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer = vec_strings!["key", "this", "is", "a", "list"];
        let error = LPushx.run(buffer, &mut data);
        assert_eq!(
            error.unwrap_err().print_it(),
            "ERR no list found with entered key".to_string()
        );
    }
}
