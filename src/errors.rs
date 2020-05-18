use crate::events::{Event, RequestEvent, ResponseEvent};
use serde_json::json;
use serde_json::Value::Null;
use std::collections::HashMap;
use uuid::Uuid;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct Error {
    err: Box<EventError>,
}

#[derive(Debug)]
pub struct EventError {
    error_type: EventErrorType,
    code: String,
    parameters: HashMap<String, String>, //todo: rever valor do mapa
}

#[derive(Debug)]
pub enum EventErrorType {
    Generic,
    BadRequest,
    Unauthorized,
    NotFound,
    Forbidden,
    UserDenied,
    ResourceDenied,
    Expired,
    Unknown(String),
}

impl EventErrorType {
    pub fn value(&self) -> &str {
        match self {
            EventErrorType::Generic => "error",
            EventErrorType::BadRequest => "badRequest",
            EventErrorType::Unauthorized => "unauthorized",
            EventErrorType::NotFound => "notFound",
            EventErrorType::Forbidden => "forbidden",
            EventErrorType::UserDenied => "userDenied",
            EventErrorType::ResourceDenied => "resourceDenied",
            EventErrorType::Expired => "expired",
            EventErrorType::Unknown(value) => value.as_str(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error type: {:?}, code: {:?}", self.err.error_type, self.err.code)
    }
}

pub fn event_not_found(event: &RequestEvent) -> ResponseEvent {
    let evt = &event.0;
    ResponseEvent(Event {
        name: String::from("eventNotFound"),
        version: 1,
        id: evt.id,
        flow_id: evt.flow_id,
        payload: json!({
            "code": "NO_EVENT_HANDLER_FOUND",
            "parameters": {
                "event": event.0.name,
                "version": event.0.version
            }
        }),
        identity: json!({}),
        auth: json!({}),
        metadata: json!({}),
    })
}

pub fn bad_protocol(err: serde_json::Error) -> ResponseEvent {
    ResponseEvent(Event {
        name: String::from("badProtocol"),
        version: 1,
        id: Uuid::new_v4(),
        flow_id: Uuid::new_v4(),
        payload: json!({
            "code": "INVALID_COMMUNICATION_PROTOCOL",
            "parameters":{
                "message": format!("{}", err)
            }
        }),
        identity: json!({}),
        auth: json!({}),
        metadata: json!({}),
    })
}

pub fn error_for(event: &RequestEvent, error: &Error) -> ResponseEvent {
    let evt = &event.0;
    let evt_error = &error.err;
    ResponseEvent(Event {
        name: format!("{}:{}", evt.name, evt_error.error_type.value()),
        version: evt.version,
        id: evt.id,
        flow_id: evt.flow_id,
        payload: json!({
            "code": evt_error.code,
            "parameters": evt_error.parameters
        }),
        identity: Null,
        auth: Null,
        metadata: Null,
    })
}
