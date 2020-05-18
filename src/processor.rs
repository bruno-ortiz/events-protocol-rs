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


