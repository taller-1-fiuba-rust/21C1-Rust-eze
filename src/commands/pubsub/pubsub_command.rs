use crate::{
    commands::{
        pubsub::{channels::Channels, numsub::Numsub},
        Runnable,
    },
    messages::redis_messages,
    native_types::ErrorStruct,
    tcp_protocol::server_redis_attributes::ServerRedisAttributes,
};

/// Gives information about the pub sub stats.
///
/// # Sub Commands
///
/// * CHANNELS: Shows all the active channels.
/// * NUMSUB: Shows all the active channels with the number of
/// subscribers.
///
/// # Error
/// Return an [ErrorStruct] if:
///
/// * User does not give a supported subcommand.
pub struct Pubsub;

impl Runnable<ServerRedisAttributes> for Pubsub {
    fn run(
        &self,
        mut buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        if !buffer.is_empty() {
            let mut subcommand = buffer.remove(0);
            subcommand.make_ascii_lowercase();
            match subcommand.as_str() {
                "channels" => Channels.run(buffer, server),
                "numsub" => Numsub.run(buffer, server),
                _ => Err(ErrorStruct::from(redis_messages::unknown_command(
                    subcommand, buffer,
                ))),
            }
        } else {
            Err(ErrorStruct::from(redis_messages::wrong_number_args_for(
                "pubsub",
            )))
        }
    }
}
