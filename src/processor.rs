use crate::errors::{bad_protocol, error_for, event_not_found};
use crate::events::{parse_event, ResponseEvent};
use crate::store::EventStore;

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
                let option_handler = self.store.handler_for(event.name.as_str(), event.version);
                if let Some(handler) = option_handler {
                    match handler.handle(&event) {
                        Ok(response) => response,
                        Err(err) => error_for(&event, &err),
                    }
                } else {
                    event_not_found(&event)
                }
            }
            Err(err) => bad_protocol(err),
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::errors::{EventError, EventErrorType};
    use crate::events::response_for;
    use crate::processor::EventProcessor;
    use crate::store::SimpleEventStore;
    use EventErrorType::{Unauthorized, BadRequest};
    use crate::errors::EventErrorType::{NotFound, Forbidden, UserDenied, ResourceDenied, Expired, Unknown};

    #[test]
    fn test_can_process_event() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |req| Ok(response_for(req, "ok")));

        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {},
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert_eq!("ok", response_event.payload.as_str().unwrap())
    }

    #[test]
    fn test_cannot_parse_event() {
        let store = SimpleEventStore::new();
        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                 }"#;

        let response_event = event_processor.process_event(raw_event);

        assert!(response_event.is_error());
        let error = response_event.get_error();

        assert_eq!(error.error_type(), "badProtocol");
        assert_eq!("INVALID_COMMUNICATION_PROTOCOL", error.value().code);
    }

    #[test]
    fn test_can_process_event_with_generic_error() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |_req| {
            Err(EventErrorType::Generic(EventError {
                code: String::from("SOME_ERROR"),
                parameters: json!({}),
            }))
        });

        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {   },
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert!(response_event.is_error());

        let error = response_event.get_error();
        assert_eq!("error", error.error_type());
        assert_eq!("SOME_ERROR", error.value().code);
    }

    #[test]
    fn test_can_process_event_with_bad_request_error() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |_req| {
            Err(BadRequest(EventError {
                code: String::from("SOME_ERROR"),
                parameters: json!({}),
            }))
        });

        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {},
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert!(response_event.is_error());

        let error = response_event.get_error();
        assert_eq!("badRequest", error.error_type());
        assert_eq!("SOME_ERROR", error.value().code);
    }

    #[test]
    fn test_can_process_event_with_unauthorized_error() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |_req| {
            Err(Unauthorized(EventError {
                code: String::from("SOME_ERROR"),
                parameters: json!({}),
            }))
        });

        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {},
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert!(response_event.is_error());

        let error = response_event.get_error();
        assert_eq!("unauthorized", error.error_type());
        assert_eq!("SOME_ERROR", error.value().code);
    }

    #[test]
    fn test_can_process_event_with_not_found_error() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |_req| {
            Err(NotFound(EventError {
                code: String::from("SOME_ERROR"),
                parameters: json!({}),
            }))
        });

        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {},
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert!(response_event.is_error());

        let error = response_event.get_error();
        assert_eq!("notFound", error.error_type());
        assert_eq!("SOME_ERROR", error.value().code);
    }

    #[test]
    fn test_can_process_event_with_forbidden_error() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |_req| {
            Err(Forbidden(EventError {
                code: String::from("SOME_ERROR"),
                parameters: json!({}),
            }))
        });

        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {},
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert!(response_event.is_error());

        let error = response_event.get_error();
        assert_eq!("forbidden", error.error_type());
        assert_eq!("SOME_ERROR", error.value().code);
    }

    #[test]
    fn test_can_process_event_with_user_denied_error() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |_req| {
            Err(UserDenied(EventError {
                code: String::from("SOME_ERROR"),
                parameters: json!({}),
            }))
        });

        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {},
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert!(response_event.is_error());

        let error = response_event.get_error();
        assert_eq!("userDenied", error.error_type());
        assert_eq!("SOME_ERROR", error.value().code);
    }

    #[test]
    fn test_can_process_event_with_resource_denied_error() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |_req| {
            Err(ResourceDenied(EventError {
                code: String::from("SOME_ERROR"),
                parameters: json!({}),
            }))
        });

        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {},
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert!(response_event.is_error());

        let error = response_event.get_error();
        assert_eq!("resourceDenied", error.error_type());
        assert_eq!("SOME_ERROR", error.value().code);
    }

    #[test]
    fn test_can_process_event_with_expired_error() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |_req| {
            Err(Expired(EventError {
                code: String::from("SOME_ERROR"),
                parameters: json!({}),
            }))
        });

        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {},
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert!(response_event.is_error());

        let error = response_event.get_error();
        assert_eq!("expired", error.error_type());
        assert_eq!("SOME_ERROR", error.value().code);
    }

    #[test]
    fn test_can_process_event_with_unknown_error() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |_req| {
            Err(Unknown(String::from("xpto"), EventError {
                code: String::from("SOME_ERROR"),
                parameters: json!({}),
            }))
        });

        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {},
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert!(response_event.is_error());

        let error = response_event.get_error();
        assert_eq!("xpto", error.error_type());
        assert_eq!("SOME_ERROR", error.value().code);
    }

    #[test]
    fn test_cannot_find_event_handler() {
        let store = SimpleEventStore::new();
        let event_processor = EventProcessor::new(Box::new(store));

        let raw_event = r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {   },
                    "auth": {}
                }"#;

        let response_event = event_processor.process_event(raw_event);

        assert_eq!(String::from("eventNotFound"), response_event.name);
        assert_eq!("NO_EVENT_HANDLER_FOUND", response_event.payload.as_object().unwrap().get("code").unwrap());
    }
}

