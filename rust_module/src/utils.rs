use std::net::TcpListener;

#[allow(dead_code)]
pub(crate) fn available_port() -> std::io::Result<u16> {
    match TcpListener::bind("0.0.0.0:0") {
        Ok(listener) => Ok(listener.local_addr().unwrap().port()),
        Err(e) => Err(e),
    }
}
