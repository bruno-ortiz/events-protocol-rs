# events-protocol-rs

[![codecov](https://codecov.io/gh/bruno-ortiz/events-protocol-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/bruno-ortiz/events-protocol-rs)

Guiabolso Events RPC protocol made in Rust Lang


This is a communication protocol agnostic event processing tool. It's the main communication protocol used at [Guiabolso](https://www.guiabolso.com.br/).

When i say "communication protocol agnostic" it means that you can use to process events coming from any source(HTTP, Streaming, Queues).

## Configuration

To configure an Event Processor you need to create an EventStore and pass it to the processor.

The EventStore is the object that will hold all the event handlers, and it will be used by the EventProcessor to select the correct handler for the given event.

```rust
let mut store = SimpleEventStore::new();

store.add("event:test", 1, |req| {
    println!("Received event with name: {}", req.name);

    Ok(response_for(req, "Event processed successfully"))
});

let event_processor = EventProcessor::new(Box::new(store));

```

Then with the processor configured you only need to pass any arbitrary event to it.

```rust
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

println!("{}", serde_json::to_string(&response_event).unwrap())

```

The code above will print:
```json
{
  "name": "event:test:response",
  "version": 1,
  "id": "f467e03c-abab-4c2f-b4cf-4871fd349c6e",
  "flowId": "cb745ef4-863b-41c4-99c7-325fe2b2b7f8",
  "payload": {
    "bar": "Bar"
  },
  "identity": {},
  "auth": {},
  "metadata": {}
}
```
##TODOS
 * describe events format
 * decribe event errors
 * Integrate with opentelemetry to generate traces automatically