use crate::native_types::ErrorStruct;

pub mod _type;
pub mod copy;
pub mod del;
pub mod rename;
//pub mod exists;
pub mod clean;

pub fn pop_value(buffer: &mut Vec<String>, name: &str) -> Result<String, ErrorStruct> {
    if let Some(value) = buffer.pop() {
        Ok(value)
    } else {
        Err(ErrorStruct::new(
            String::from("ERR"),
            "wrong number of arguments for ".to_owned() + "\'" + name + "\'" + " command",
        ))
    }
}

pub fn no_more_values(buffer: &[String], name: &str) -> Result<(), ErrorStruct> {
    if buffer.is_empty() {
        Ok(())
    } else {
        Err(ErrorStruct::new(
            String::from("ERR"),
            "wrong number of arguments for ".to_owned() + "\'" + name + "\'" + " command",
        ))
    }
}

fn parse_integer(value: String) -> Result<isize, ErrorStruct> {
    if let Ok(index) = value.parse::<isize>() {
        Ok(index)
    } else {
        Err(ErrorStruct::new(
            String::from("ERR"),
            String::from("value is not an integer or out of range"),
        ))
    }
}
