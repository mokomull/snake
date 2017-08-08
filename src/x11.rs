use std::io;
use tokio_core::reactor::Handle;
use tokio_uds::UnixStream;
use tokio_io::AsyncRead;
use tokio_io::io::{ReadHalf, WriteHalf, read_exact, write_all};

// TODO: un-pub
pub struct X11Client {
    pub write: WriteHalf<UnixStream>,
    pub read: ReadHalf<UnixStream>,
}

impl X11Client {
    pub fn connect_unix(handle: &Handle, display: usize) -> io::Result<Self> {
        let path = format!("/tmp/.X11-unix/X{}", display);
        let socket = UnixStream::connect(path, handle)?;

        let split = socket.split();
        Ok(X11Client {
            read: split.0,
            write: split.1,
        })
    }
}
