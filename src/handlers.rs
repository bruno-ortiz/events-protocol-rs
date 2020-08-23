use crate::errors::EventErrorType;
use crate::events::{RequestEvent, ResponseEvent};

pub trait EventHandler {
    fn handle(&self, event: &RequestEvent) -> Result<ResponseEvent, EventErrorType>;
}

pub struct FnForwardHandler<T: Fn(&RequestEvent) -> Result<ResponseEvent, EventErrorType>> {
    fn_handler: T,
}

impl<T: Fn(&RequestEvent) -> Result<ResponseEvent, EventErrorType>> FnForwardHandler<T> {
    pub fn new(handler: T) -> FnForwardHandler<T> {
        FnForwardHandler {
            fn_handler: handler,
        }
    }
}

impl<T: Fn(&RequestEvent) -> Result<ResponseEvent, EventErrorType>> EventHandler for FnForwardHandler<T> {
    fn handle(&self, event: &RequestEvent) -> Result<ResponseEvent, EventErrorType> {
        (self.fn_handler)(event)
    }
}