use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;
use crate::errors::{EventErrorType, EventError};


#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Debug)]
pub struct RequestEvent(pub Event);

#[derive(Debug)]
pub struct ResponseEvent(pub Event);

impl ResponseEvent {
    pub fn is_success(&self) -> bool {
        self.0.name.ends_with(":response")
    }

    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    pub fn get_error(&self) -> EventErrorType {
        if !self.is_error() {
            panic!("Cannot get error when the response is success")
        }
        let event = &self.0;
        let last_separator_idx = event.name
            .rfind(":")
            .and_then(|idx| Some(idx + 1))
            .unwrap_or(0);

        let error_type = &event.name[last_separator_idx..];

        return EventErrorType::new(error_type, EventError {
            code: event.payload.get("code").unwrap().as_str().unwrap().into(),
            parameters: event.payload.get("parameters").unwrap().clone(),
        });
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
) -> ResponseEvent {
    let evt = &event.0;
    ResponseEvent(Event {
        name: format!("{}:{}", evt.name, "response"),
        version: evt.version,
        id: Uuid::from(evt.id),
        flow_id: Uuid::from(evt.flow_id),
        payload: serde_json::to_value(payload).unwrap(), //todo: tratar esse result
        identity: json!({}),
        auth: json!({}),
        metadata: json!({}),
    })
}
