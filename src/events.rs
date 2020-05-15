use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use serde_json::Value::Null;
use uuid::Uuid;
use std::fmt::Display;
use serde::export::Formatter;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Event {
    name: String,
    version: u16,
    id: Uuid,
    flow_id: Uuid,
    payload: Value,
    identity: Value,
    auth: Value,
    metadata: Value,
}

pub struct RequestEvent(Event);

pub struct ResponseEvent(Event);

impl Display for ResponseEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self.0).unwrap())
    }
}

pub struct Error {
    err: Box<EventError>,
}

pub struct EventError {
    error_type: EventErrorType,
    code: String,
    parameters: HashMap<String, String>, //todo: rever valor do mapa
}

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
            EventErrorType::Unknown(value) => value.as_str()
        }
    }
}

fn parse_event(payload: &str) -> Result<RequestEvent, serde_json::Error> {
    match serde_json::from_str::<Event>(payload) {
        Ok(event) => {
            //todo: write a event validator to validate that its a valid event
            Ok(RequestEvent(event))
        }
        Err(err) => Err(err)
    }
}

fn response_for<T: Serialize>(event: &RequestEvent, payload: T) -> Result<ResponseEvent, Error> {
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

fn event_not_found(event: &RequestEvent) -> ResponseEvent {
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

fn bad_protocol(err: serde_json::Error) -> ResponseEvent {
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

fn error_for(event: &RequestEvent, error: &Error) -> ResponseEvent {
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

mod handlers {
    use crate::events::{Error, RequestEvent, ResponseEvent};

    pub trait EventHandler {
        fn handle(&self, event: &RequestEvent) -> Result<ResponseEvent, Error>;
    }

    pub struct FnForwardHandler<T: Fn(&RequestEvent) -> Result<ResponseEvent, Error>> {
        fn_handler: T
    }

    impl<T: Fn(&RequestEvent) -> Result<ResponseEvent, Error>> FnForwardHandler<T> {
        pub fn new(handler: T) -> FnForwardHandler<T> {
            FnForwardHandler { fn_handler: handler }
        }
    }

    impl<T: Fn(&RequestEvent) -> Result<ResponseEvent, Error>> EventHandler for FnForwardHandler<T> {
        fn handle(&self, event: &RequestEvent) -> Result<ResponseEvent, Error> {
            (self.fn_handler)(event)
        }
    }
}


mod store {
    use std::collections::HashMap;

    use crate::events::{Error, RequestEvent, ResponseEvent};
    use crate::events::handlers::{EventHandler, FnForwardHandler};

    pub trait EventStore {
        fn handler_for<'a>(&'a self, event_name: &'a str, version: u16) -> Option<&'a Box<dyn EventHandler>>;
    }

    pub struct SimpleEventStore {
        handlers: HashMap<(&'static str, u16), Box<dyn EventHandler>>
    }

    impl SimpleEventStore {
        pub fn new() -> Self {
            SimpleEventStore { handlers: HashMap::new() }
        }

        pub fn add<T>(&mut self, name: &'static str, version: u16, handler: T) where T: Fn(&RequestEvent) -> Result<ResponseEvent, Error> + 'static {
            self.handlers.insert((name, version), Box::new(FnForwardHandler::new(handler)));
        }
    }

    impl EventStore for SimpleEventStore {
        fn handler_for<'a>(&'a self, event_name: &'a str, version: u16) -> Option<&'a Box<dyn EventHandler>> {
            self.handlers.get(&(event_name, version))
        }
    }
}

mod processor {
    use crate::events::{bad_protocol, error_for, event_not_found, parse_event, ResponseEvent};
    use crate::events::store::EventStore;

    pub struct EventProcessor {
        store: Box<dyn EventStore>,
    }

    impl EventProcessor {
        pub fn new(store: Box<dyn EventStore>) -> Self {
            EventProcessor {
                store,
            }
        }

        pub fn process_event(&self, payload: &str) -> ResponseEvent {
            match parse_event(payload) {
                Ok(event) => {
                    let evt = &event.0;
                    let option_handler = self.store.handler_for(evt.name.as_str(), evt.version);
                    if let Some(handler) = option_handler {
                        match handler.handle(&event) {
                            Ok(response) => response,
                            Err(err) => error_for(&event, &err)
                        }
                    } else {
                        event_not_found(&event)
                    }
                }
                Err(err) => bad_protocol(err)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::events::processor::EventProcessor;
    use crate::events::response_for;
    use crate::events::store::SimpleEventStore;

    #[test]
    fn test_can_add_event_handler() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |req| response_for(req, "ok"));

        let processor = EventProcessor::new(Box::new(store));
        let response_event = processor.process_event(r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {   },
                    "auth": {}
                }"#);
        println!("response: {}", response_event)
    }
}