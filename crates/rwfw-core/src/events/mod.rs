pub mod handler;

use rwfw_shared::events::AppEvent;
use std::sync::RwLock;

pub type EventHandler = Box<dyn Fn(&AppEvent) + Send + Sync>;

pub struct EventSubscription {
    pub event_name: String,
    pub handler: EventHandler,
}

pub struct EventBus {
    subscriptions: RwLock<Vec<EventSubscription>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscriptions: RwLock::new(Vec::new()),
        }
    }

    pub fn subscribe(&self, subscription: EventSubscription) {
        let mut subs = self.subscriptions.write().unwrap();
        subs.push(subscription);
    }

    pub fn emit(&self, event: &AppEvent) {
        let event_name = match event {
            AppEvent::UserCreated { .. } => "UserCreated",
            AppEvent::UserLoggedIn { .. } => "UserLoggedIn",
            AppEvent::PostPublished { .. } => "PostPublished",
            AppEvent::PostDeleted { .. } => "PostDeleted",
        };

        tracing::debug!(event = %event_name, "Dispatching event");

        let subs = self.subscriptions.read().unwrap();
        for sub in subs.iter() {
            if sub.event_name == event_name || sub.event_name == "*" {
                (sub.handler)(event);
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
