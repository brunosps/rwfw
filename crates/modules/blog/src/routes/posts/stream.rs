use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing;
use rwfw_core::app::AppState;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

/// GET /blog/posts/stream - Server-Sent Events of Turbo Stream fragments
/// (e.g. a new post card prepended to the list in real time).
async fn get(State(state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.broadcaster.subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(|message| match message {
        Ok(html) => Some(Ok(Event::default().data(html))),
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
