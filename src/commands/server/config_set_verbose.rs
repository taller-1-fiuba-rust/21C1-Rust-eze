use crate::{
    commands::{check_empty, Runnable},
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
    tcp_protocol::server::ServerRedisAtributes,
};
pub struct ConfigSetVerbose;

impl Runnable<ServerRedisAtributes> for ConfigSetVerbose {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAtributes,
    ) -> Result<String, ErrorStruct> {
        check_empty(&buffer, "config set verbose")?;

        // No empty! Ok first unwrap!
        match buffer.get(0).unwrap().parse::<usize>() {
            Ok(level) => {
                server.change_verbose(level)?;
                Ok(RSimpleString::encode(redis_messages::ok()))
            }
            Err(_) => Err(ErrorStruct::new(
                String::from("ERR"),
                String::from("Invalid verbose level received."),
            )),
        }
    }
}