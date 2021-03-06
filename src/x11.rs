use byteorder::{BigEndian, ByteOrder};
use futures::stream::unfold;
use futures::{Future, Stream};
use std::io::{self, Error};
use tokio::io::{read_exact, write_all, AsyncRead, ReadHalf, WriteHalf};
use tokio::net::UnixStream;

use x11_client::*;

pub fn connect_unix(
    display: usize,
) -> impl Future<Item = (ServerInit, X11Client, X11Events), Error = Error> {
    let path = format!("/tmp/.X11-unix/X{}", display);
    UnixStream::connect(path).and_then(|socket| {
        let (read, write) = socket.split();
        let client = X11Client { write };
        let events = X11Events { read };

        client.write_init().and_then(move |client| {
            events
                .read_init()
                .map(move |(events, server_init)| (server_init, client, events))
        })
    })
}

pub struct X11Client {
    write: WriteHalf<UnixStream>,
}

impl X11Client {
    // TODO: I have no idea why the compiler wants 'static here
    fn write_all<T>(self, buf: T) -> impl Future<Item = Self, Error = Error>
    where
        T: AsRef<[u8]> + Send + 'static,
    {
        let write = self.write;

        write_all(write, buf).map(move |(socket, _)| Self { write: socket })
    }

    fn write_init(self) -> impl Future<Item = Self, Error = Error> {
        let client_init: Vec<_> = ClientInit::new().into();
        self.write_all(client_init)
    }

    // TODO: x and y are INT16, not CARD16
    #[allow(clippy::too_many_arguments)] // since these all come from X11, not me :)
    pub fn create_window(
        self,
        depth: u8,
        window: u32,
        parent: u32,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        border_width: u16,
        class: u16,
        visual: u32,
    ) -> impl Future<Item = Self, Error = Error> {
        self.write_all(
            CreateWindow::new(
                depth,
                window,
                parent,
                x,
                y,
                width,
                height,
                border_width,
                class,
                visual,
            )
            .as_bytes(),
        )
    }

    pub fn map_window(self, window: u32) -> impl Future<Item = Self, Error = Error> {
        self.write_all(MapWindow::new(window).as_bytes())
    }

    pub fn change_wm_name(
        self,
        window: u32,
        name: &str,
    ) -> impl Future<Item = Self, Error = Error> {
        self.write_all(ChangeWmName::new(window, name.into()).as_bytes())
    }

    pub fn create_gc(
        self,
        gc_id: u32,
        window: u32,
        color: u32,
    ) -> impl Future<Item = Self, Error = Error> {
        self.write_all(CreateGc::new(gc_id, window, color).as_bytes())
    }

    pub fn poly_fill_rectangle(
        self,
        drawable: u32,
        gc: u32,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> impl Future<Item = Self, Error = Error> {
        self.write_all(PolyFillRectangle::new(drawable, gc, x, y, width, height).as_bytes())
    }
}

pub struct X11Events {
    read: ReadHalf<UnixStream>,
}

impl X11Events {
    fn read_exact<T>(self, buf: T) -> impl Future<Item = (Self, T), Error = Error>
    where
        T: AsMut<[u8]> + Send + 'static,
    {
        let read = self.read;

        read_exact(read, buf).map(move |(socket, result)| (Self { read: socket }, result))
    }

    fn read_init(self) -> impl Future<Item = (Self, ServerInit), Error = Error> {
        let server_init_prefix = vec![0 as u8; 8];

        // TODO: the destructuring of the server response length really belongs
        // in x11_client, since it depends on the byteorder that it serialized
        // into ClientInit.
        self.read_exact(server_init_prefix)
            .and_then(|(events, mut server_init_prefix)| {
                assert_eq!(1, server_init_prefix[0]);
                let length = BigEndian::read_u16(&server_init_prefix[6..8]);
                events.read_exact(vec![0 as u8; (length * 4) as usize]).map(
                    move |(events, mut server_init_rest)| {
                        let mut server_init_data = Vec::new();
                        server_init_data.append(&mut server_init_prefix);
                        server_init_data.append(&mut server_init_rest);

                        let server_init =
                            ServerInit::from_stream(&mut io::Cursor::new(server_init_data))
                                .unwrap();
                        (events, server_init)
                    },
                )
            })
    }

    pub fn into_stream(self) -> impl Stream<Item = Event, Error = Error> {
        unfold(self.read, |read| {
            let f = read_exact(read, [0 as u8; 32])
                .map(|(socket, result)| (Event::from_bytes(&result), socket));
            Some(f)
        })
    }
}
