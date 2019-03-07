use byteorder::{BigEndian, ByteOrder};
use futures::stream::unfold;
use futures::{Future, Stream};
use std::io::{self, Result};
use tokio::io::{read_exact, write_all, AsyncRead, ReadHalf, WriteHalf};
use tokio::net::UnixStream;

use x11_client::*;

pub async fn connect_unix(display: usize) -> Result<(ServerInit, X11Client, X11Events)> {
    let path = format!("/tmp/.X11-unix/X{}", display);
    let socket = await!(UnixStream::connect(path))?;
    let (read, write) = socket.split();
    let mut client = X11Client { write };
    let mut events = X11Events { read };

    await!(client.write_init())?;
    let server_init = await!(events.read_init())?;
    Ok((server_init, client, events))
}

pub struct X11Client {
    write: WriteHalf<UnixStream>,
}

impl X11Client {
    // TODO: I have no idea why the compiler wants 'static here
    async fn write_all<T>(&mut self, buf: T) -> std::io::Result<()>
    where
        T: AsRef<[u8]> + Send + 'static,
    {
        await!(write_all(&mut self.write, buf))?;
        Ok(())
    }

    async fn write_init(&mut self) -> Result<()> {
        let client_init: Vec<_> = ClientInit::new().into();
        await!(self.write_all(client_init))
    }

    // TODO: x and y are INT16, not CARD16
    #[allow(clippy::too_many_arguments)] // since these all come from X11, not me :)
    pub async fn create_window(
        &mut self,
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
    ) -> std::io::Result<()> {
        await!(self.write_all(
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
        ))
    }

    pub async fn map_window(&mut self, window: u32) -> std::io::Result<()> {
        await!(self.write_all(MapWindow::new(window).as_bytes()))
    }

    pub async fn change_wm_name<'a>(&'a mut self, window: u32, name: &'a str) -> Result<()> {
        await!(self.write_all(ChangeWmName::new(window, name.into()).as_bytes()))
    }

    pub async fn create_gc(&mut self, gc_id: u32, window: u32, color: u32) -> Result<()> {
        await!(self.write_all(CreateGc::new(gc_id, window, color).as_bytes()))
    }

    pub async fn poly_fill_rectangle(
        &mut self,
        drawable: u32,
        gc: u32,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> Result<()> {
        await!(self.write_all(PolyFillRectangle::new(drawable, gc, x, y, width, height).as_bytes()))
    }
}

pub struct X11Events {
    read: ReadHalf<UnixStream>,
}

impl X11Events {
    async fn read_exact<T>(&mut self, buf: T) -> Result<T>
    where
        T: AsMut<[u8]> + Send + 'static,
    {
        let (_, result) = await!(read_exact(&mut self.read, buf))?;
        Ok(result)
    }

    async fn read_init(&mut self) -> Result<ServerInit> {
        let server_init_prefix = vec![0 as u8; 8];

        // TODO: the destructuring of the server response length really belongs
        // in x11_client, since it depends on the byteorder that it serialized
        // into ClientInit.
        let mut server_init_prefix = await!(self.read_exact(server_init_prefix))?;
        assert_eq!(1, server_init_prefix[0]);
        let length = BigEndian::read_u16(&server_init_prefix[6..8]);
        let mut server_init_rest = await!(self.read_exact(vec![0 as u8; (length * 4) as usize]))?;
        let mut server_init_data = Vec::new();
        server_init_data.append(&mut server_init_prefix);
        server_init_data.append(&mut server_init_rest);

        let server_init = ServerInit::from_stream(&mut io::Cursor::new(server_init_data)).unwrap();
        Ok(server_init)
    }

    pub fn into_stream(self) -> impl Stream<Item = Event, Error = std::io::Error> {
        unfold(self.read, |read| {
            let f = read_exact(read, [0 as u8; 32])
                .map(|(socket, result)| (Event::from_bytes(&result), socket));
            Some(f)
        })
    }
}
