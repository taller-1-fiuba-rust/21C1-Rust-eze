use crate::commands::Runnable;
use crate::communication::log_messages::LogMessage;
use crate::messages::redis_messages;
use crate::native_types::ErrorStruct;
use crate::tcp_protocol::client_atributes::client_fields::ClientFields;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;

use self::notifier::Notifier;

pub mod client_atributes;
pub mod client_handler;
pub mod client_list;
pub mod command_delegator;
pub mod command_subdelegator;
pub mod commands_map;
pub mod listener_processor;
pub mod notifier;
pub mod runnables_map;
pub mod server;
pub mod server_redis_attributes;

pub type RawCommand = (Vec<String>, Sender<Response>, Arc<Mutex<ClientFields>>);
pub type RawCommandTwo = Option<Arc<BoxedCommand<Arc<Mutex<ClientFields>>>>>;
pub type BoxedCommand<T> = Box<dyn Runnable<T> + Send + Sync>;
pub type Response = Result<String, ErrorStruct>;

#[allow(dead_code)]
fn get_command(command_input_user: &[String]) -> String {
    let mut command_type = command_input_user[0].clone();
    if command_type.contains("config") & command_input_user.len().eq(&3) {
        command_type = command_type.to_owned() + " " + &command_input_user[1].to_string();
        if command_input_user[1].to_string().contains("set") {
            command_type.push(' ');
            command_type.push_str(&command_input_user[2].to_string());
        }
    }
    command_type
}

pub fn close_thread(
    thread: Option<JoinHandle<Result<(), ErrorStruct>>>,
    name: &str,
    notifer: Notifier,
) -> Result<(), ErrorStruct> {
    if let Some(handle) = thread {
        handle
            .join()
            .map_err(|_| {
                let _ = notifer.send_log(LogMessage::theard_panic(name)); // I'm not interested ... I retired with the forced Shutdown!
                ErrorStruct::from(redis_messages::thread_panic(name))
            })?
            .and_then(|_| {
                notifer.send_log(LogMessage::theard_closed(name))?;
                Ok(())
            })
    } else {
        Ok(())
    }
}
