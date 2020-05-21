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
                let evt = &event.0;
                let option_handler = self.store.handler_for(evt.name.as_str(), evt.version);
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
    use crate::processor::EventProcessor;
    use crate::store::SimpleEventStore;
    use crate::events::response_for;

    #[test]
    fn test_can_process_event() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |req| response_for(req, "ok"));

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

        assert_eq!("ok", response_event.0.payload.as_str().unwrap())
    }

    #[test]
    fn test_cannot_find_event_handler() {
        let mut store = SimpleEventStore::new();
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

        assert_eq!(String::from("eventNotFound"), response_event.0.name);
        assert_eq!("NO_EVENT_HANDLER_FOUND", response_event.0.payload.as_object().unwrap().get("code").unwrap());
    }
}

