use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Event {
    AccountCreated,
    TransactionPosted,
    TransactionReversed,
}

pub struct EventBus {
    handlers: Vec<Box<dyn Fn(&Event)>>,
}

impl EventBus {
    pub fn new() -> EventBus {
        EventBus {
            handlers: Vec::new()
        }
    }

    pub fn subscribe(&mut self, handler: Box<dyn Fn(&Event)>) {
        self.handlers.push(handler)
    }

    pub fn publish(&self, event: &Event) {
        for handler in self.handlers.iter() {
            handler(event)
        }
    }
}

