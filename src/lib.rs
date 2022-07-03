use std::borrow::Cow;
use std::fmt::Display;
use std::time::Instant;

use tracing::{error, error_span, field, info, info_span, warn, warn_span};
use trillium::{async_trait, HeaderName};
use trillium::{Conn, Handler, Info};

struct TracerRun;

#[async_trait]
impl Handler for Tracer {
    async fn run(&self, conn: Conn) -> Conn {
        let path = path(&conn);
        let method = conn.method();
        let ip = ip(&conn);
        let headers = conn.headers();
        let hn: HeaderName = "Accept".into();
        let k = headers.get_str(hn);
        info!(k);

        // for i in headers.iter() {
        //     info!("{}", i.0.into_owned());
        //     info!("{:?}", i.1);
        // }

        info_span!("Request", http.method = ?method, ip = ?ip, path = %path, headers = ?headers)
            .in_scope(|| {
                info!("received");
                conn.with_state(TracerRun)
            })
    }
    async fn init(&mut self, info: &mut Info) {
        info!("Starting server");
        info!(server = ?info.server_description(), socker_addr = ?info.tcp_socket_addr().unwrap());
    }

    async fn before_send(&self, mut conn: Conn) -> Conn {
        let response_len = conn.response_len().unwrap_or_default();
        let response_time = response_time(&conn).to_string();

        // TODO: If the conn.status is not set by a route, should we set to NotFound by default if
        // the user doesn't set the default handler in trillium?
        let status = conn.status(); //.unwrap_or_else(|| Status::NotFound);

        if conn.state::<TracerRun>().is_some() {
            conn.inner_mut().after_send(move |s| {
                let response_span = info_span!("Response", http.status_code = field::Empty, http.duration = ?response_time, http.response_len = ?response_len);
                response_span.in_scope(|| {
                    if let Some(status) = status {
                        response_span.record("http.status_code", &(status as u32));
                        if s.is_success() {
                            if status.is_server_error() {
                                error!("Internal Server error");
                            } else if status.is_client_error() {
                                warn_span!("Client error").in_scope(|| warn!("sent"))
                            } else {
                                info!("sent")
                            }
                        } else {
                            error_span!("Internal error").in_scope(|| warn!("cannot be sent"))
                        }
                    } else {
                        // This place shouldn't be reached ideally
                        // Set the default status in the router
                        error_span!("Internal error").in_scope(|| 
                            error!("status not set, set the default status")
                        );
                    }

                });
            });
        }
        conn
    }
}

struct ResponseTimeOutput(Instant);
impl Display for ResponseTimeOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", Instant::now() - self.0))
    }
}

fn response_time(conn: &Conn) -> ResponseTimeOutput {
    ResponseTimeOutput(conn.inner().start_time())
}

fn ip(conn: &Conn) -> Cow<'static, str> {
    match conn.inner().peer_ip() {
        Some(peer) => format!("{:?}", peer).into(),
        None => "-".into(),
    }
}

fn path(conn: &Conn) -> Cow<'static, str> {
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
