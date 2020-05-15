use std::collections::HashMap;

use crate::errors::Error;
use crate::handlers::{EventHandler, FnForwardHandler};
use crate::events::{RequestEvent, ResponseEvent};

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