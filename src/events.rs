use std::collections::HashMap;
use std::fmt::Display;

use serde::Deserialize;
use serde::export::Formatter;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use serde_json::Value::Null;
use uuid::Uuid;
use crate::errors::Error;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub name: String,
    pub version: u16,
    pub id: Uuid,
    pub flow_id: Uuid,
    pub payload: Value,
    pub identity: Value,
    pub auth: Value,
    pub metadata: Value,
}

pub struct RequestEvent(pub Event);

pub struct ResponseEvent(pub Event);

impl Display for ResponseEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self.0).unwrap())
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


#[cfg(test)]
mod tests {
    use crate::store::SimpleEventStore;
    use crate::events::response_for;
    use crate::processor::EventProcessor;

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