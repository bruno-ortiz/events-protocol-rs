use crate::errors::Error;
use crate::events::{RequestEvent, ResponseEvent};
use std::ops::Deref;

pub trait EventHandler {
    fn handle(&self, event: &RequestEvent) -> Result<ResponseEvent, Error>;
}

pub struct FnForwardHandler<T: Fn(&RequestEvent) -> Result<ResponseEvent, Error>> {
    fn_handler: T,
}

impl<T: Fn(&RequestEvent) -> Result<ResponseEvent, Error>> FnForwardHandler<T> {
    pub fn new(handler: T) -> FnForwardHandler<T> {
        FnForwardHandler {
            fn_handler: handler,
        }
    }
}

impl<T: Fn(&RequestEvent) -> Result<ResponseEvent, Error>> EventHandler for FnForwardHandler<T> {
    fn handle(&self, event: &RequestEvent) -> Result<ResponseEvent, Error> {
        (self.fn_handler)(event)
    }
}