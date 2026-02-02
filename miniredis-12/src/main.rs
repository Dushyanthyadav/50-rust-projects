use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod commands;
mod protocol;
mod storage;

use commands::Command;
use protocol::decode;
use storage::Db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Intialize shared Database
    let db = Db::new();

    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("mini-redis listening on 127.0.0.1:6379");

    loop {
        let (socket, _) = listener.accept().await?;

        let db_handle = db.clone();

        tokio::spawn(async move {
            process_connection(socket, db_handle).await;
        });
    }
}

async fn process_connection(mut socket: TcpStream, db: Db) {
    let mut buffer = BytesMut::with_capacity(4096);

    loop {
        let _n = match socket.read_buf(&mut buffer).await {
            Ok(n) if n == 0 => return,
            Ok(n) => n,
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                return;
            }
        };

        loop {
            match decode(&mut buffer) {
                Ok(Some(frame)) => {
                    let response = match Command::from_resp(frame) {
                        Ok(cmd) => cmd.execute(&db),
                        Err(err_msg) => protocol::RespType::Error(err_msg),
                    };

                    if let Err(e) = socket.write_all(&response.serialize()).await {
                        eprintln!("failed to write to socket; err = {:?}", e);
                        return;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    eprintln!("Protocol error: {:?}. Closing connection.", e);
                    return;
                }
            }
        }
    }
}
