use crate::{
    commands::keys::{no_more_values, parse_integer, pop_value},
    commands::Runnable,
    database::Database,
    messages::redis_messages,
    native_types::{error_severity::ErrorSeverity, ErrorStruct, RInteger, RedisType},
};

use std::sync::{Arc, Mutex, MutexGuard};
pub struct Clean;

impl Runnable<Arc<Mutex<Database>>> for Clean {
    /// Touches n database elements (see TOUCH command for a deeper understanding) forcing
    /// key expiration (if it's configured). If more than 25% was expired, the process is
    /// repeated.
    ///
    /// # Return value
    /// * [String] _encoded_ in [RInteger](crate::native_types::integer::RInteger): number of total keys expired.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty, or received with a number of elements
    /// different than 1.
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
        let argument = pop_value(&mut buffer, "clean")?;

        no_more_values(&buffer, "clean")?;
        let iterations = parse_integer(argument)?;

        let mut continue_cleaning = true;
        let mut total_of_expired_keys = 0;

        while continue_cleaning {
            let amount_of_expired_keys: isize = touch_n_random_keys(&iterations, &mut database);

            if amount_of_expired_keys <= (iterations / 4) {
                continue_cleaning = false;
            }

            total_of_expired_keys += amount_of_expired_keys;
        }

        Ok(RInteger::encode(total_of_expired_keys))
    }
}

fn touch_n_random_keys(n: &isize, database: &mut MutexGuard<Database>) -> isize {
    let mut expired_keys: isize = 0;
    for _ in 0..*n {
        if let Some(key) = database.random_key() {
            let _ = database
                .touch(&key)
                .map(|is_expired| is_expired.then(|| expired_keys += 1));
        }
    }
    expired_keys
}

#[cfg(test)]
mod test_clean {
    use crate::commands::create_notifier;

    use super::*;
    use crate::database::TypeSaved;
    use crate::vec_strings;
    use std::collections::VecDeque;
    use std::thread::sleep;
    use std::time::Duration;

    fn load_database(database: &mut Database) {
        database.insert(
            "Agustin".to_string(),
            TypeSaved::String("Firmapaz".to_string()),
        );
        database.insert(
            "Martina".to_string(),
            TypeSaved::String("Panetta".to_string()),
        );
        database.insert(
            "Federico".to_string(),
            TypeSaved::String("Pacheco".to_string()),
        );

        let mut profes: VecDeque<String> = VecDeque::new();
        profes.push_back("Pablo".to_string());
        profes.push_back("Matias".to_string());
        profes.push_back("Uriel".to_string());
        database.insert("profes".to_string(), TypeSaved::List(profes));
    }

    #[test]
    //#[ignore = "Long test"]
    fn test_01_cleaning_some_keys() {
        let (notifier, _log_rcv, _cmd_rcv) = create_notifier();
        let mut database = Database::new(notifier);

        load_database(&mut database);

        database.set_ttl("Agustin", 2).unwrap();
        database.set_ttl("Federico", 5).unwrap();

        sleep(Duration::new(5, 0));

        let command = vec_strings!["3"];
        let mut c_database = Arc::new(Mutex::new(database));
        let mut response = Clean.run(command, &mut c_database).unwrap();
        response.remove(0);
        response.pop();
        response.pop();
        let expired_keys = parse_integer(response).unwrap();
        assert!(expired_keys <= 2);
    }
}
