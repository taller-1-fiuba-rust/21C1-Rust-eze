use crate::vec_strings;
use std::collections::HashSet;

pub fn available_commands() -> HashSet<String> {
    let available_commands_list: Vec<String> = vec_strings![
        "decrby",
        "del",
        "expire",
        "exists",
        "get",
        "getset",
        "incrby",
        "keys",
        "lindex",
        "llen",
        "lpop",
        "lpush",
        "lrange",
        "lrem",
        "lset",
        "mget",
        "mset",
        "rename",
        "rpop",
        "rpush",
        "sadd",
        "scard",
        "set",
        "shutdown",
        "sismember",
        "smembers",
        "sort",
        "srem",
        "ttl",
        "type"
    ];
    let available_commands_set: HashSet<String> = available_commands_list
        .iter()
        .map(|member| member.to_string())
        .collect();
    available_commands_set
}
