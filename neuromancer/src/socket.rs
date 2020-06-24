use tokio::net::TcpStream;

pub struct Socket {
    inner: TcpStream,
}
