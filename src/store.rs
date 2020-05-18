use std::collections::HashMap;

use crate::errors::Error;
use crate::events::{RequestEvent, ResponseEvent};
use crate::handlers::{EventHandler, FnForwardHandler};

pub trait EventStore {
    fn handler_for<'a>(
        &'a self,
        event_name: &'a str,
        version: u16,
    ) -> Option<&'a Box<dyn EventHandler>>;
}

pub struct SimpleEventStore {
    handlers: HashMap<(&'static str, u16), Box<dyn EventHandler>>,
}

impl SimpleEventStore {
    pub fn new() -> Self {
        SimpleEventStore {
            handlers: HashMap::new(),
        }
    }

    pub fn add<T>(&mut self, name: &'static str, version: u16, handler: T)
        where T: Fn(&RequestEvent) -> Result<ResponseEvent, Error> + 'static {
        self.handlers.insert((name, version), Box::new(FnForwardHandler::new(handler)));
    }
}

impl EventStore for SimpleEventStore {
    fn handler_for<'a>(
        &'a self,
        event_name: &'a str,
        version: u16,
    ) -> Option<&'a Box<dyn EventHandler>> {
        self.handlers.get(&(event_name, version))
    }
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use crate::events::{parse_event, response_for};
    use crate::store::{EventStore, SimpleEventStore};
    use serde::export::fmt::Debug;
    use crate::handlers::EventHandler;
    use serde::export::Formatter;

    #[test]
    fn test_can_add_event_handler() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |req| response_for(req, "ok"));

        let req = parse_event(r#"{
                    "name": "event:test",
                    "version": 1,
                    "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
                    "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
                    "payload": {},
                    "metadata": {},
                    "identity": {   },
                    "auth": {}
                }"#).unwrap();

        let option = store.handler_for("event:test", 1);
        assert_that(&option).is_some();
        let response = option.unwrap().handle(&req).unwrap();

        assert_that(&response.0.payload.as_str().unwrap()).is_equal_to("ok")
    }


    impl Debug for dyn EventHandler {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Handler")
        }
    }
}
