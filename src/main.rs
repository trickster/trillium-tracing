use smol::Timer;
use std::time::Duration;
use trillium::{Conn, Handler};
use trillium_router::{Router, RouterConnExt};
use trillium_tracing::Tracer;

fn router() -> impl Handler {
    Router::new()
        .get("/hello", "hi")
        // .post("/", |mut conn: Conn| async move {
        //     Timer::after(Duration::from_millis(200)).await;
        //     let body = conn.request_body_string().await.unwrap();
        //     conn.ok(format!("request body: {}", body))
        // })
        .get("/hello/:planet", |conn: Conn| async move {
            Timer::after(Duration::from_millis(200)).await;
            if let Some(planet) = conn.param("planet") {
                let response = format!("hello, {}", planet);
                conn.ok(response)
            } else {
                conn
            }
        })
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    trillium_smol::run((Tracer::new(), router()));
}
