use crate::events::{Event, RequestEvent, ResponseEvent};
use serde_json::{json, Value};
use serde_json::Value::Null;
use uuid::Uuid;
use std::fmt::{Debug, Display, Formatter};
use crate::errors::EventErrorType::{Generic, Unknown, BadRequest, Unauthorized, NotFound, Forbidden, UserDenied, ResourceDenied, Expired};

#[derive(Debug, Clone)]
pub struct EventError {
    pub code: String,
    pub parameters: Value,
}

#[derive(Debug)]
pub enum EventErrorType {
    Generic(EventError),
    BadRequest(EventError),
    Unauthorized(EventError),
    NotFound(EventError),
    Forbidden(EventError),
    UserDenied(EventError),
    ResourceDenied(EventError),
    Expired(EventError),
    Unknown(String),
}

impl EventErrorType {
    pub fn new(error_type: &str, error: EventError) -> Self {
        match error_type {
            "error" => Generic(error),
            "badRequest" => BadRequest(error),
            "unauthorized" => Unauthorized(error),
            "notFound" => NotFound(error),
            "forbidden" => Forbidden(error),
            "userDenied" => UserDenied(error),
            "resourceDenied" => ResourceDenied(error),
            "expired" => Expired(error),
            _ => Unknown(String::from(error_type)),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            EventErrorType::Generic(_) => "error",
            EventErrorType::BadRequest(_) => "badRequest",
            EventErrorType::Unauthorized(_) => "unauthorized",
            EventErrorType::NotFound(_) => "notFound",
            EventErrorType::Forbidden(_) => "forbidden",
            EventErrorType::UserDenied(_) => "userDenied",
            EventErrorType::ResourceDenied(_) => "resourceDenied",
            EventErrorType::Expired(_) => "expired",
            EventErrorType::Unknown(value) => value.as_str(),
        }
    }

    pub fn value(&self) -> EventError {
        match self {
            EventErrorType::Generic(err) => err.clone(),
            EventErrorType::BadRequest(err) => err.clone(),
            EventErrorType::Unauthorized(err) => err.clone(),
            EventErrorType::NotFound(err) => err.clone(),
            EventErrorType::Forbidden(err) => err.clone(),
            EventErrorType::UserDenied(err) => err.clone(),
            EventErrorType::ResourceDenied(err) => err.clone(),
            EventErrorType::Expired(err) => err.clone(),
            EventErrorType::Unknown(_value) => EventError {
                code: String::from("UNKNOWN_ERROR"),
                parameters: json!({}),
            },
        }
    }
}

impl Display for EventErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error type: {:?}, code: {:?}", self.name(), self.value().code)
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

pub fn error_for(event: &RequestEvent, error: &EventErrorType) -> ResponseEvent {
    let evt = &event.0;
    let evt_error = error.value();
    ResponseEvent(Event {
        name: format!("{}:{}", evt.name, error.name()),
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
