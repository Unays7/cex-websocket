use futures_util::{StreamExt, SinkExt, stream::SplitStream, stream::SplitSink};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio::task;
use std::error::Error;

pub struct WebSocket {
    url: String,
    read: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    name: String,
}

impl WebSocket {
    pub fn new(url: String, name: String) -> Self {
        WebSocket {
            url,
            read: None,
            write: None,
            name,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Connecting to: {}", self.url);
        let (ws_stream, _) = connect_async(&self.url).await?;
        println!("Connected :)");
        let (write, read) = ws_stream.split();
        self.write = Some(write);
        self.read = Some(read);
        Ok(())
    }

    pub async fn start_stream(&mut self, request: String) -> Result<(), Box<dyn Error>> {
        let json_request = json!({
            "method": "SUBSCRIBE",
            "params": [
                request
            ],
            "id": 1
        });
        let msg = Message::Text(json_request.to_string());

        if let Some(write) = &mut self.write {
            write.send(msg).await?;
        }
        Ok(())
    }

    pub async fn print_stream(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(read) = &mut self.read {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        println!("{} received a message: {}", self.name, text);
                    }
                    Ok(Message::Close(e)) => {
                        if let Some(e) = e {
                            println!("Closed with reason: {}", e.reason);
                        } else {
                            println!("Closed without reason");
                        }
                        break;
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut btcusdt_aggTrade_client = WebSocket::new(
        "wss://stream.binance.com:443/ws".to_string(),
        "btcusdt_aggTrade_client".to_string(),
    );
    btcusdt_aggTrade_client.connect().await?;
    btcusdt_aggTrade_client.start_stream("btcusdt@aggTrade".to_string()).await?;

    let btcusdt_agg_trade_handle = task::spawn(async move {
        if let Err(e) = btcusdt_aggTrade_client.print_stream().await {
            eprintln!("btcusdt_aggTrade_client error: {}", e);
        }
    });

    let mut btcusd_bookTicker_client = WebSocket::new(
        "wss://stream.binance.com:443/ws".to_string(),
        "btcusd_bookTicker_client".to_string(),
    );
    btcusd_bookTicker_client.connect().await?;
    btcusd_bookTicker_client.start_stream("btcusdt@bookTicker".to_string()).await?;

    let btcusd_book_ticker_handle = task::spawn(async move {
        if let Err(e) = btcusd_bookTicker_client.print_stream().await {
            eprintln!("btcusd_bookTicker_client error: {}", e);
        }
    });

    btcusdt_agg_trade_handle.await?;
    btcusd_book_ticker_handle.await?;

    Ok(())
}
