use smol::Timer;
use std::time::Duration;
use trillium::{Conn, Handler, Status};
use trillium_router::{Router, RouterConnExt};
use trillium_tracing::Tracer;

fn router() -> impl Handler {
    Router::new()
        .get("/hello", "hi")
        .get("/error", |conn: Conn| async move {
            conn.with_body("Custom error page!")
                .with_status(Status::BadGateway)
                .halt()
        })
        .get("/hello/:planet", |conn: Conn| async move {
            Timer::after(Duration::from_millis(200)).await;
            if let Some(planet) = conn.param("planet") {
                let response = format!("hello, {}", planet);
                conn.ok(response)
            } else {
                conn
            }
        })
        // This is for matching all other routes, using Status::NotFound
        .get("/*", |conn: Conn| async move {
            conn.with_body("Wrong place - default 404")
                .with_status(Status::NotFound)
                .halt()
        })
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    trillium_smol::run((Tracer::new(), router()));
}
