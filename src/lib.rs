use std::borrow::Cow;
use std::fmt::Display;
use std::time::Instant;

use tracing::{error, error_span, field, info, info_span, warn, warn_span};
use trillium::async_trait;
use trillium::{Conn, Handler, Info, Status};

struct TracerRun;

#[async_trait]
impl Handler for Tracer {
    async fn run(&self, conn: Conn) -> Conn {
        let path = path(&conn);
        let method = conn.method();
        let ip = ip(&conn);
        info_span!("Request", http.method = ?method, ip = ?ip, path = ?path).in_scope(|| {
            info!("received");
            conn.with_state(TracerRun)
        })
        // conn.with_state(TracerRun)
    }
    async fn init(&mut self, info: &mut Info) {
        info!("Starting server");
        info!(server = ?info.server_description(), socker_addr = ?info.tcp_socket_addr().unwrap());
    }

    async fn before_send(&self, mut conn: Conn) -> Conn {
        let response_len = conn.response_len().unwrap_or_default();
        let response_time = response_time(&conn).to_string();
        let status = conn.status(); //.unwrap_or_else(|| Status::NotFound);

        if conn.state::<TracerRun>().is_some() {
            conn.inner_mut().after_send(move |s| {
                info_span!("Response", http.status_code = field::Empty, http.duration = ?response_time, http.response_len = ?response_len).in_scope(|| {
                    if let Some(status) = status {
                        if s.is_success() {
                            if status.is_server_error() {
                                let span = error_span!("Internal error", error = field::Empty);
                                span.record("error", &"server error");
                            } else if status.is_client_error() {
                                warn_span!("Client error").in_scope(|| warn!("sent"));
                            } else {
                                // info_span!("Request", http.method = ?method, ip = ?ip, path = ?path).in_scope(|| {
                                info!("sent")
                                // });
                            }
                        } else {
                            let span = error_span!("Internal error", error = field::Empty);
                            span.in_scope(|| error!("cannot be sent"));
                            // error_span!("Internal error").in_scope(|| warn!("sending error"));
                        }
                    } else {
                        // warn!("sent response with no status");
                        let span = error_span!("Server error", error = field::Empty);
                        span.record("error", &"sent response with no status");
                        error_span!("Internal error").in_scope(|| error!("status not set, set the default status"));
                    }

                });
            });
        }
        conn
    }
}

pub struct ResponseTimeOutput(Instant);
impl Display for ResponseTimeOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", Instant::now() - self.0))
    }
}

pub fn response_time(conn: &Conn) -> ResponseTimeOutput {
    ResponseTimeOutput(conn.inner().start_time())
}

pub fn ip(conn: &Conn) -> Cow<'static, str> {
    match conn.inner().peer_ip() {
        Some(peer) => format!("{:?}", peer).into(),
        None => "-".into(),
    }
}

pub fn path(conn: &Conn) -> Cow<'static, str> {
    let p = conn.inner().path();
    format!("{p:?}").into()
}

#[derive(Debug, Default, Clone)]
pub struct Tracer;

impl Tracer {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
