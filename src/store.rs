use std::collections::HashMap;

use crate::errors::Error;
use crate::events::{RequestEvent, ResponseEvent};
use crate::handlers::{EventHandler, FnForwardHandler};
use std::ops::Deref;

pub trait EventStore {
    fn handler_for(
        &self,
        event_name: &str,
        version: u16,
    ) -> Option<&dyn EventHandler>;
}

pub struct SimpleEventStore<'a> {
    handlers: HashMap<(String, u16), Box<dyn EventHandler + 'a>>,
}

impl<'a> SimpleEventStore<'a> {
    pub fn new() -> Self {
        SimpleEventStore {
            handlers: HashMap::new(),
        }
    }

    pub fn add<T>(&mut self, name: &str, version: u16, handler: T)
        where T: Fn(&RequestEvent) -> Result<ResponseEvent, Error> + 'a {
        self.handlers.insert((String::from(name), version), Box::new(FnForwardHandler::new(handler)));
    }
}

impl<'a> EventStore for SimpleEventStore<'a> {
    fn handler_for(
        &self,
        event_name: &str,
        version: u16,
    ) -> Option<&dyn EventHandler>{
        match self.handlers.get(&(String::from(event_name), version)) {
            Some(handler) => Some(handler.deref()),
            None => None,
        }
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
