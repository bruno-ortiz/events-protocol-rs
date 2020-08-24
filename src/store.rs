use std::collections::HashMap;

use crate::events::{RequestEvent, ResponseEvent};
use crate::handlers::{EventHandler, FnForwardHandler};
use std::ops::Deref;
use crate::errors::EventErrorType;

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
        where T: Fn(&RequestEvent) -> Result<ResponseEvent, EventErrorType> + 'a {
        self.handlers.insert((String::from(name), version), Box::new(FnForwardHandler::new(handler)));
    }
}

impl<'a> EventStore for SimpleEventStore<'a> {
    fn handler_for(
        &self,
        event_name: &str,
        version: u16,
    ) -> Option<&dyn EventHandler> {
        match self.handlers.get(&(String::from(event_name), version)) {
            Some(handler) => Some(handler.deref()),
            None => None,
        }
    }
}

impl<'a> Default for SimpleEventStore<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::events::{parse_event, response_for};
    use crate::store::{EventStore, SimpleEventStore};

    #[test]
    fn test_can_add_event_handler() {
        let mut store = SimpleEventStore::new();
        store.add("event:test", 1, |req| Ok(response_for(req, "ok")));

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
        assert!(option.is_some(), "Could no find event handler");

        let handler = option.unwrap();
        let result = handler.handle(&req);

        assert!(result.is_ok(), "Error executing handler. Error: {:?}", result);
        let response = result.unwrap();
        assert_eq!("ok", response.payload.as_str().unwrap())
    }

    #[test]
    fn test_cannot_find_event_handler() {
        let store = SimpleEventStore::new();
        let option = store.handler_for("event:test", 1);
        assert!(option.is_none());
    }
}
