use byteorder::{BigEndian, ByteOrder};
use futures::compat::{AsyncRead01CompatExt, Future01CompatExt};
use futures::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use futures::stream::unfold;
use futures::Stream;
use std::io::{self, Result};
use tokio::net::UnixStream;

use x11_client::*;

pub async fn connect_unix(display: usize) -> Result<(ServerInit, X11Client, X11Events)> {
    let path = format!("/tmp/.X11-unix/X{}", display);
    let socket = UnixStream::connect(path).compat().await?;
    let (read, write) = socket.compat().split();
    let mut client = X11Client { write };
    let mut events = X11Events { read };

    client.write_init().await?;
    let server_init = events.read_init().await?;
    Ok((server_init, client, events))
}

pub struct X11Client {
    write: WriteHalf<futures::compat::Compat01As03<UnixStream>>,
}

impl X11Client {
    // TODO: I have no idea why the compiler wants 'static here
    async fn write_all<T>(&mut self, buf: T) -> std::io::Result<()>
    where
        T: AsRef<[u8]> + Send + 'static,
    {
        self.write.write_all(buf.as_ref()).await?;
        Ok(())
    }

    async fn write_init(&mut self) -> Result<()> {
        let client_init: Vec<_> = ClientInit::new().into();
        self.write_all(client_init).await
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
        .await
    }

    pub async fn map_window(&mut self, window: u32) -> std::io::Result<()> {
        self.write_all(MapWindow::new(window).as_bytes()).await
    }

    pub async fn change_wm_name<'a>(&'a mut self, window: u32, name: &'a str) -> Result<()> {
        self.write_all(ChangeWmName::new(window, name.into()).as_bytes())
            .await
    }

    pub async fn create_gc(&mut self, gc_id: u32, window: u32, color: u32) -> Result<()> {
        self.write_all(CreateGc::new(gc_id, window, color).as_bytes())
            .await
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
        self.write_all(PolyFillRectangle::new(drawable, gc, x, y, width, height).as_bytes())
            .await
    }
}

pub struct X11Events {
    read: ReadHalf<futures::compat::Compat01As03<UnixStream>>,
}

impl X11Events {
    async fn read_exact<T>(&mut self, mut buf: T) -> Result<T>
    where
        T: AsMut<[u8]>,
    {
        self.read.read_exact(buf.as_mut()).await?;
        Ok(buf)
    }

    async fn read_init(&mut self) -> Result<ServerInit> {
        let server_init_prefix = vec![0 as u8; 8];

        // TODO: the destructuring of the server response length really belongs
        // in x11_client, since it depends on the byteorder that it serialized
        // into ClientInit.
        let mut server_init_prefix = self.read_exact(server_init_prefix).await?;
        assert_eq!(1, server_init_prefix[0]);
        let length = BigEndian::read_u16(&server_init_prefix[6..8]);
        let mut server_init_rest = self
            .read_exact(vec![0 as u8; (length * 4) as usize])
            .await?;
        let mut server_init_data = Vec::new();
        server_init_data.append(&mut server_init_prefix);
        server_init_data.append(&mut server_init_rest);

        let server_init = ServerInit::from_stream(&mut io::Cursor::new(server_init_data)).unwrap();
        Ok(server_init)
    }

    pub fn into_stream(self) -> impl Stream<Item = Event> {
        Box::pin(unfold(self.read, read_event))
    }
}

async fn read_event<T>(mut read: T) -> Option<(Event, T)>
where
    T: futures::io::AsyncRead + std::marker::Unpin,
{
    let mut buf = [0 as u8; 32];
    read.read_exact(&mut buf).await.ok()?;
    Some((Event::from_bytes(&buf), read))
}
