use std::io;
use futures::Future;
use tokio_core::reactor::Handle;
use tokio_uds::UnixStream;
use tokio_io::{AsyncRead, IoFuture};
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

    // TODO: I have no idea why the compiler wants 'static here
    fn read_exact<T>(self, buf: T) -> IoFuture<(Self, T)>
    where
        T: AsMut<[u8]> + Send + 'static,
    {
        read_exact(self.read, buf)
            .map(|(socket, result)| {
                (
                    Self {
                        read: socket,
                        write: self.write,
                    },
                    result,
                )
            })
            .boxed()
    }

    fn write_all<T>(self, buf: T) -> IoFuture<Self>
    where
        T: AsRef<[u8]> + Send + 'static
    {
        write_all(self.write, buf)
            .map(|(socket, _)| {
                Self {
                    read: self.read,
                    write: socket,
                }
            })
            .boxed()
    }
}
