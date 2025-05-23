use std::fmt::Debug;
use std::str::FromStr;
use std::time::Instant;
use tokio::sync::mpsc::Sender;
use tokio::task;
use tracing::{Event, Level, Subscriber};
use tracing::field::{Field, Visit};
use tracing_subscriber::{EnvFilter, Layer, Registry};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::util::{SubscriberInitExt, TryInitError};

#[derive(Clone, Debug)]
pub struct Log {
    pub level: Level,
    pub message: String,
    pub time: Instant,
}

#[derive(Clone)]
pub struct AppLogLayer {
    sender: Sender<Log>,
}

impl<S: Subscriber> Layer<S> for AppLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let message = String::new();
        
        struct Visitor(String);
        
        impl Visit for Visitor {
            fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
                if field.name().eq_ignore_ascii_case("message") {
                    self.0 += &format!("{value:?}");
                }
            }
        }
        
        let mut visitor = Visitor(message);
        
        event.record(&mut visitor);
        
        let tx = self.sender.clone();
        
        let level = *event.metadata().level();
        
        let message = visitor.0;
        
        task::spawn_blocking(move || tx.blocking_send(Log { level, message, time: Instant::now() }));
    }
}

pub fn init(tx: Sender<Log>) -> Result<(), TryInitError> {
    Registry::default()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or(
                    EnvFilter::from_str("warn,bluetooth_timeout=debug")
                        .expect("failed to set the default env filter")
                )
        )
        .with(AppLogLayer { sender: tx })
        .try_init()
}
