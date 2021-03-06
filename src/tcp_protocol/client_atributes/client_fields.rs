use crate::messages::redis_messages::broken_state;
use crate::messages::redis_messages::not_valid_executor;
use crate::messages::redis_messages::not_valid_monitor;
use crate::messages::redis_messages::not_valid_pubsub;
use crate::messages::redis_messages::unexpected_behaviour;

use crate::native_types::ErrorStruct;
use crate::tcp_protocol::client_atributes::status::Status;
use crate::tcp_protocol::runnables_map::RunnablesMap;
use crate::tcp_protocol::RawCommandTwo;
use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::Mutex;

use std::net::SocketAddrV4;

/// Contains the atributes of one client.
/// Its behaviour depends on the client status.
pub struct ClientFields {
    map: Option<RunnablesMap<Arc<Mutex<ClientFields>>>>,
    status: Status,
    subscriptions: HashSet<String>,
    pub address: SocketAddrV4,
}

impl ClientFields {
    /// Return a new instance of the Client Fields
    ///
    /// # Return value
    /// [ClientFields]
    ///
    pub fn new(address: SocketAddrV4) -> ClientFields {
        ClientFields {
            map: Some(RunnablesMap::<Arc<Mutex<ClientFields>>>::executor()),
            status: Status::Executor,
            subscriptions: HashSet::new(),
            address,
        }
    }

    /// Returns the address of the client.
    ///
    /// # Return value
    /// [String]: the address of the client.
    ///
    pub fn get_addr(&self) -> String {
        self.address.clone().to_string()
    }

    /// Replace the status of the client with a new given one
    ///
    /// # Return value
    /// [Status]: the last status.
    ///
    pub fn replace_status(&mut self, new_status: Status) -> Status {
        let old_status = self.status.replace(new_status);
        self.update_map();
        old_status
    }

    /// Returns a wrapped reference of the client's status.
    ///
    /// # Return value
    /// [Option]: the wrapped status.
    ///
    pub fn status(&self) -> Option<&Status> {
        Some(&self.status)
    }

    /// Returns true if the client is subscripted to any channel.
    ///
    /// # Return value
    /// [bool]
    ///
    pub fn is_subscripted_to(&self, channel: &str) -> bool {
        self.subscriptions.contains(channel)
    }

    /// Returns true if the client is dead.
    ///
    /// # Return value
    /// [bool]
    ///
    pub fn is_dead(&self) -> bool {
        self.status.eq(&Status::Dead)
    }

    /// Check if the client could execute a given command.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Client is in monitor or dead status.
    ///
    pub fn is_allowed_to(&self, command: &str) -> Result<(), ErrorStruct> {
        match self.status {
            Status::Executor => Ok(()),
            Status::Subscriber => self
                .map
                .as_ref()
                .ok_or_else(|| ErrorStruct::from(broken_state()))?
                .contains_key(command)
                .then(|| ())
                .ok_or_else(|| ErrorStruct::from(not_valid_pubsub())),
            _ => Err(ErrorStruct::from(not_valid_monitor())),
        }
    }

    /// Returns the requested runnable.
    ///
    /// # Return value
    /// [RawCommandTwo]: The runnable
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The client is not in a valid status to execute the command.
    pub fn review_command(&self, command: &[String]) -> Result<RawCommandTwo, ErrorStruct> {
        match self.status {
            Status::Executor => self.rc_case_executor(command),
            Status::Subscriber => self.rc_case_subscriber(command),
            Status::Monitor => Err(ErrorStruct::new(
                not_valid_monitor().get_prefix(),
                not_valid_monitor().get_message(),
            )),
            Status::Dead => panic!(),
        }
    }

    /// Returns true if the client is in monitor mode.
    ///
    /// # Return value
    /// [bool]
    ///
    pub fn is_monitor_notificable(&self) -> bool {
        self.status == Status::Monitor
    }

    fn rc_case_subscriber(&self, command: &[String]) -> Result<RawCommandTwo, ErrorStruct> {
        Some(
            self.map
                .as_ref()
                .ok_or_else(|| ErrorStruct::from(broken_state()))?
                .get(command.get(0).unwrap()),
        )
        .ok_or_else(|| ErrorStruct::from(not_valid_pubsub()))
    }

    fn rc_case_executor(&self, command: &[String]) -> Result<RawCommandTwo, ErrorStruct> {
        Some(
            self.map
                .as_ref()
                .ok_or_else(|| ErrorStruct::from(broken_state()))?
                .get(command.get(0).unwrap()),
        )
        .ok_or_else(|| ErrorStruct::from(not_valid_executor()))
    }

    fn update_map(&mut self) {
        self.map = self.status.update_map();
    }

    /// Add the given channels to the subscription list.
    ///
    /// # Return value
    /// [isize]: The number of channels which the client
    /// was not subscribed.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The client is not in a valid status to execute the command.
    pub fn add_subscriptions(&mut self, channels: Vec<String>) -> Result<isize, ErrorStruct> {
        match self.status {
            Status::Executor => Ok(self.as_case_executor(channels)),
            Status::Subscriber => Ok(self.as_case_subscriber(channels)),
            _ => Err(ErrorStruct::from(unexpected_behaviour(
                "Dead client (or monitor) is trying to execute invalid command",
            ))),
        }
    }

    fn as_case_executor(&mut self, channels: Vec<String>) -> isize {
        let added = self.add_channels(channels);
        self.replace_status(Status::Subscriber);
        added
    }

    fn as_case_subscriber(&mut self, channels: Vec<String>) -> isize {
        self.add_channels(channels)
    }

    /// Remove the given channels of the subscription list.
    ///
    /// # Return value
    /// [isize]: The number of channels which has been removed.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The client is not in a valid status to execute the command.
    pub fn remove_subscriptions(&mut self, channels: Vec<String>) -> Result<isize, ErrorStruct> {
        match &self.status {
            Status::Executor => Ok(0),
            Status::Subscriber => Ok(self.rs_case_subscriber(channels)),
            _ => Err(ErrorStruct::new(
                unexpected_behaviour(
                    "Dead client (or monitor) is trying to execute invalid command",
                )
                .get_prefix(),
                unexpected_behaviour(
                    "Dead client (or monitor) is trying to execute invalid command",
                )
                .get_message(),
            )),
        }
    }

    fn rs_case_subscriber(&mut self, channels: Vec<String>) -> isize {
        if channels.is_empty() {
            self.status.replace(Status::Executor);
            self.subscriptions.clear();
        } else {
            let _removed = self.remove_channels(channels);
            if self.subscriptions.is_empty() {
                self.replace_status(Status::Executor);
            }
        }
        self.subscriptions.len() as isize
    }

    fn add_channels(&mut self, new_channels: Vec<String>) -> isize {
        for channel in new_channels.iter() {
            self.subscriptions.insert(String::from(channel));
        }
        self.subscriptions.len() as isize
    }

    fn remove_channels(&mut self, new_channels: Vec<String>) -> isize {
        for channel in new_channels.iter() {
            self.subscriptions.remove(channel);
        }
        self.subscriptions.len() as isize
    }

    /// Return the details of the client atributes.
    ///
    /// # Return value
    /// [isize]: The number of channels which the client
    /// was not subscribed.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * The client is not in a valid status to execute the command.
    pub fn get_detail(&self) -> String {
        format!(
            "Client: {:?} -- Status: {:?} -- Subscriptions: {:?}",
            self.address.to_string(),
            self.status,
            self.subscriptions
        )
    }
}

impl Default for ClientFields {
    fn default() -> ClientFields {
        ClientFields::new(SocketAddrV4::new(Ipv4Addr::new(1, 0, 0, 1), 8080))
    }
}

#[cfg(test)]
mod test_client_status {

    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_01_initial_state() {
        let status = ClientFields::new(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
        assert_eq!(status.status(), Some(&Status::Executor));
    }

    #[test]
    fn test_02_add_subscriptions() {
        let mut status = ClientFields::new(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
        let added = status.add_subscriptions(vec!["telefe".to_string(), "trece".to_string()]);
        assert_eq!(added.unwrap(), 2);
        assert_eq!(status.status(), Some(&Status::Subscriber));
    }

    #[test]
    fn test_03_remove_not_all_subscriptions() {
        let mut status = ClientFields::new(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
        let added = status.add_subscriptions(vec![
            "telefe".to_string(),
            "trece".to_string(),
            "martina".to_string(),
        ]);
        assert_eq!(added.unwrap(), 3);
        assert_eq!(status.status(), Some(&Status::Subscriber));

        let removed =
            status.remove_subscriptions(vec!["telefe".to_string(), "martina".to_string()]);
        assert_eq!(removed.unwrap(), 1);
        assert_eq!(status.status(), Some(&Status::Subscriber));
    }

    #[test]
    fn test_04_remove_all_subscriptions() {
        let mut status = ClientFields::new(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
        let added = status.add_subscriptions(vec![
            "telefe".to_string(),
            "trece".to_string(),
            "martina".to_string(),
        ]);
        assert_eq!(added.unwrap(), 3);
        assert_eq!(status.status(), Some(&Status::Subscriber));

        let removed = status.remove_subscriptions(vec![
            "telefe".to_string(),
            "trece".to_string(),
            "martina".to_string(),
        ]);
        assert_eq!(removed.unwrap(), 0);
        assert_eq!(status.status(), Some(&Status::Executor));
    }

    #[test]
    fn test_05_remove_all_subscriptions_by_default_empty_vec() {
        let mut status = ClientFields::new(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
        let added = status.add_subscriptions(vec![
            "telefe".to_string(),
            "trece".to_string(),
            "martina".to_string(),
        ]);
        assert_eq!(added.unwrap(), 3);
        assert_eq!(status.status(), Some(&Status::Subscriber));

        let removed = status.remove_subscriptions(vec![]);
        assert_eq!(removed.unwrap(), 0);
        assert_eq!(status.status(), Some(&Status::Executor));
    }
}
