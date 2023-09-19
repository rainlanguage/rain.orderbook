use crate::cli::serve::websocket::MyWs;
use actix::Addr;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use tracing::Event;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::Context;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

use super::serve::websocket::TraceEvent;

lazy_static! {
    pub static ref WS_CONNECTION: Arc<Mutex<Option<Addr<MyWs>>>> = Arc::new(Mutex::new(None));
}

pub fn initialize_tracing() {
    let fmt_layer = tracing_subscriber::fmt::Layer::new().with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(WebSocketLayer.with_filter(LevelFilter::DEBUG))
        .init();
}

struct WebSocketLayer;

impl<S: tracing::Subscriber> Layer<S> for WebSocketLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        if let Some(conn) = &*WS_CONNECTION.lock().unwrap() {
            // Send the formatted event to the WebSocket connection.
            let message = format!("{:?}", event);
            conn.do_send(TraceEvent(message));
        }
    }
}
