use std::fs;

use anyhow::Context;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::{UnixListener, UnixStream},
    sync::broadcast,
};

use crate::event_stream::StreamContext;

pub struct Socket;

impl Socket {
    pub async fn run(path: &str, context: StreamContext) -> anyhow::Result<()> {
        let _ = fs::remove_file(path);

        let file_path = std::path::Path::new(path);
        let file_dir = std::path::Path::parent(file_path)
            .expect(format!("Cannot determine socket directory: {path}").as_str());

        fs::create_dir_all(
            file_dir
                .to_str()
                .expect(format!("Cannot determine socket directory: {path}").as_str()),
        )
        .expect(format!("Cannot create socket directory for path: {path}").as_str());

        let listener =
            UnixListener::bind(path).with_context(|| format!("bind unix socket {path}"))?;

        loop {
            let Ok((stream, _)) = listener.accept().await else {
                continue;
            };
            let context = context.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(stream, context).await {
                    println!("stream client error: {e:#}");
                }
            });
        }
    }

    async fn handle_client(stream: UnixStream, context: StreamContext) -> anyhow::Result<()> {
        let (_, write) = stream.into_split();
        let mut writer = BufWriter::new(write);

        let mut rx = context.subscribe().unwrap();

        loop {
            match rx.recv().await {
                Ok(event) => {
                    let Ok(mut line) = serde_json::to_vec(&event) else {
                        continue;
                    };

                    line.push(b'\n');

                    let Ok(_) = writer.write_all(&line).await else {
                        continue;
                    };
                    let Ok(_) = writer.flush().await else {
                        continue;
                    };
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return Ok(());
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
            }
        }
    }
}
