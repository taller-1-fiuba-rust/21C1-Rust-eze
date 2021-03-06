use crate::tcp_protocol::client_atributes::client_fields::ClientFields;
use std::mem;
use std::sync::Arc;
use std::sync::Mutex;

use crate::tcp_protocol::runnables_map::RunnablesMap;

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Executor,
    Subscriber,
    Monitor,
    Dead,
}

impl Status {
    /// Replace the status of the client with a new given one
    ///
    /// # Return value
    /// [Status]: the last status.
    ///
    pub fn replace(&mut self, new_status: Status) -> Status {
        mem::replace(self, new_status)
    }

    /// Update the runnables map of the client, depending on
    /// the status.
    ///
    /// # Return value
    /// [RunnablesMap].
    ///
    pub fn update_map(&self) -> Option<RunnablesMap<Arc<Mutex<ClientFields>>>> {
        match self {
            Self::Executor => Some(RunnablesMap::<Arc<Mutex<ClientFields>>>::executor()),
            Self::Subscriber => Some(RunnablesMap::<Arc<Mutex<ClientFields>>>::subscriber()),
            _ => None,
        }
    }
}
