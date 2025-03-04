//! Run with
//!
//! ```not_rust
//! cargo run -p example-sse
//! ```

use axum::{
    extract::TypedHeader,
    handler::get,
    http::StatusCode,
    response::sse::{Event, Sse},
    Router,
};
use futures::stream::{self, Stream};
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tokio_stream::StreamExt as _;
use tower_http::{services::ServeDir, trace::TraceLayer};

#[tokio::main]
async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "example_sse=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();

    let static_files_service = axum::service::get(
        ServeDir::new("examples/sse/assets").append_index_html_on_directories(true),
    )
    .handle_error(|error: std::io::Error| {
        Ok::<_, std::convert::Infallible>((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", error),
        ))
    });

    // build our application with a route
    let app = Router::new()
        .nest("/", static_files_service)
        .route("/sse", get(sse_handler))
        .layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    // A `Stream` that repeats an event every second
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream)
}
