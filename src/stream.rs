use crate::*;

pub struct StreamMessage {
    session_id: String,
    destination: String,
    contents: Vec<u8>,
}
