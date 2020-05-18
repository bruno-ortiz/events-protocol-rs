use std::fmt::Display;

use serde::Deserialize;
use serde::export::Formatter;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

use crate::errors::Error;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub name: String,
    pub version: u16,
    pub id: Uuid,
    pub flow_id: Uuid,
    pub payload: Value,
    pub identity: Value,
    pub auth: Value,
    pub metadata: Value,
}

pub struct RequestEvent(pub Event);

pub struct ResponseEvent(pub Event);

impl Display for ResponseEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self.0).unwrap())
    }
}

pub fn parse_event(payload: &str) -> Result<RequestEvent, serde_json::Error> {
    match serde_json::from_str::<Event>(payload) {
        Ok(event) => {
            //todo: write a event validator to validate that its a valid event
            Ok(RequestEvent(event))
        }
        Err(err) => Err(err),
    }
}

pub fn response_for<T: Serialize>(
    event: &RequestEvent,
    payload: T,
) -> Result<ResponseEvent, Error> {
    let evt = &event.0;
    Ok(ResponseEvent(Event {
        name: format!("{}:{}", evt.name, "response"),
        version: evt.version,
        id: evt.id,
        flow_id: evt.flow_id,
        payload: serde_json::to_value(payload).unwrap(), //todo: tratar esse result
        identity: json!({}),
        auth: json!({}),
        metadata: json!({}),
    }))
}
