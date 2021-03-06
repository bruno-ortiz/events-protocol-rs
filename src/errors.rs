use crate::errors::EventErrorType::{
    BadRequest, Expired, Forbidden, Generic, NotFound, ResourceDenied, Unauthorized, Unknown,
    UserDenied,
};
use crate::events::{RequestEvent, ResponseEvent};
use serde_json::{json, Value};
use std::fmt::{Debug, Display, Formatter};
use uuid::Uuid;

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
    Unknown(String, EventError),
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
            _ => Unknown(String::from(error_type), error),
        }
    }

    pub fn error_type(&self) -> &str {
        match self {
            Generic(_) => "error",
            BadRequest(_) => "badRequest",
            Unauthorized(_) => "unauthorized",
            NotFound(_) => "notFound",
            Forbidden(_) => "forbidden",
            UserDenied(_) => "userDenied",
            ResourceDenied(_) => "resourceDenied",
            Expired(_) => "expired",
            Unknown(value, _) => value.as_str(),
        }
    }

    pub fn value(&self) -> EventError {
        match self {
            Generic(err) => err.clone(),
            BadRequest(err) => err.clone(),
            Unauthorized(err) => err.clone(),
            NotFound(err) => err.clone(),
            Forbidden(err) => err.clone(),
            UserDenied(err) => err.clone(),
            ResourceDenied(err) => err.clone(),
            Expired(err) => err.clone(),
            Unknown(_value, err) => err.clone(),
        }
    }
}

impl Display for EventErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error type: {:?}, code: {:?}",
            self.error_type(),
            self.value().code
        )
    }
}

pub fn event_not_found(event: &RequestEvent) -> ResponseEvent {
    ResponseEvent {
        name: String::from("eventNotFound"),
        version: 1,
        id: event.id,
        flow_id: event.flow_id,
        payload: json!({
            "code": "NO_EVENT_HANDLER_FOUND",
            "parameters": {
                "event": event.name,
                "version": event.version
            }
        }),
        identity: json!({}),
        auth: json!({}),
        metadata: json!({}),
    }
}

pub fn bad_protocol(err: serde_json::Error) -> ResponseEvent {
    ResponseEvent {
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
    }
}

pub fn error_for(event: &RequestEvent, error: &EventErrorType) -> ResponseEvent {
    let evt_error = error.value();
    ResponseEvent {
        name: format!("{}:{}", event.name, error.error_type()),
        version: event.version,
        id: event.id,
        flow_id: event.flow_id,
        payload: json!({
            "code": evt_error.code,
            "parameters": evt_error.parameters
        }),
        identity: json!({}),
        auth: json!({}),
        metadata: json!({}),
    }
}
