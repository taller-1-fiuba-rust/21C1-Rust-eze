use super::{no_more_values, pop_value};
use crate::commands::Runnable;
use crate::database::{Database, TypeSaved};
use crate::messages::redis_messages;
use crate::native_types::bulk_string::RBulkString;
use crate::native_types::error::ErrorStruct;
use crate::native_types::error_severity::ErrorSeverity;
use crate::native_types::redis_type::RedisType;
use std::sync::{Arc, Mutex};

pub struct Getdel;

impl Runnable<Arc<Mutex<Database>>> for Getdel {
    /// Get the **value** of **key** and delete the **key**.
    /// This command is similar to GET, except for the fact that it also deletes the key on success (if and only if the key's value type is a string).
    ///
    /// # Return value
    /// [String] _encoded_ in [RBulkString]: the **value of **key**, **nil** when **key** does not exist
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The key's value type isn't a string.
    /// * The buffer [Vec]<[String]> more than one elements is received or empty.
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
        let key = pop_value(&mut buffer)?;
        no_more_values(&buffer, "getdel")?;

        if let Some(value) = database.get(&key) {
            match value {
                TypeSaved::String(_) => (),
                _ => {
                    return Err(ErrorStruct::new(
                        String::from("ERR"),
                        String::from("key provided is not from string"),
                    ))
                }
            }
        } else {
            return Ok(RBulkString::encode("(nil)".to_string()));
        }

        if let Some(value) = database.remove(&key) {
            match value {
                TypeSaved::String(value) => Ok(RBulkString::encode(value)),
                _ => Err(ErrorStruct::new(
                    String::from("ERR"),
                    String::from("key provided is not from string"),
                )),
            }
        } else {
            Ok(RBulkString::encode("(nil)".to_string()))
        }
    }
}

#[cfg(test)]
pub mod test_getdel {
    use crate::commands::create_notifier;

    use super::*;
    use crate::{
        database::{Database, TypeSaved},
        vec_strings,
    };

    #[test]
    fn test_01_getdel_of_an_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec_strings!["key"];
        let encoded = Getdel.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), "$5\r\nvalue\r\n".to_string());
        assert_eq!(data.lock().unwrap().get("key"), None);
    }

    #[test]
    fn test_02_getdel_of_a_non_existing_key() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));
        let buffer = vec_strings!["key"];
        let encoded = Getdel.run(buffer, &mut data);

        assert_eq!(encoded.unwrap(), "$-1\r\n".to_string());
        assert_eq!(data.lock().unwrap().get("key"), None);
    }

    #[test]
    fn test_03_wrong_number_of_arguments() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut data = Arc::new(Mutex::new(Database::new(notifier)));

        data.lock()
            .unwrap()
            .insert("key".to_string(), TypeSaved::String("value".to_string()));

        let buffer = vec_strings!["key", "ahre", "mas", "argumentos"];
        let encoded = Getdel.run(buffer, &mut data);
        match encoded {
            Ok(_value) => {}
            Err(error) => assert_eq!(
                error.print_it(),
                "ERR wrong number of arguments for 'getdel' command"
            ),
        }
    }
}
