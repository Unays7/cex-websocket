use futures_util::{StreamExt, SinkExt, stream::SplitStream, stream::SplitSink};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio::task;

pub struct WebSocket {
    url: String,
    read: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    name : String,
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

    pub async fn connect(&mut self) {
        println!("Connecting to: {}", self.url);
        let (ws_stream, _) = connect_async(&self.url).await.expect("Failed to connect");
        println!("Connected :)");
        let (write, read) = ws_stream.split();
        self.write = Some(write);
        self.read = Some(read);
    }

    pub async fn start_stream(&mut self, request: String) {
        let json_request = json!({
            "method": "SUBSCRIBE",
            "params": [
                request
            ],
            "id": 1
        });
        let msg = Message::Text(json_request.to_string());

        if let Some(write) = &mut self.write {
            write.send(msg).await.expect("Failed to send msg");
        }
    }

    pub async fn print_stream(&mut self) {
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
    }
}

#[tokio::main]
async fn main() {

    let mut btcusdt_aggTrade_client= WebSocket::new("wss://stream.binance.com:443/ws".to_string(), "btcusdt_aggTrade_client".to_string());
    btcusdt_aggTrade_client.connect().await;
    btcusdt_aggTrade_client.start_stream("btcusdt@aggTrade".to_string()).await;

    let btcusdt_agg_trade_handle = task::spawn(async move {
        btcusdt_aggTrade_client.print_stream().await;
    });


    let mut btcusd_bookTicker_client = WebSocket::new("wss://stream.binance.com:443/ws".to_string(), "btcusd_bookTicker_client".to_string());
    btcusd_bookTicker_client.connect().await;
    btcusd_bookTicker_client.start_stream("btcusdt@bookTicker".to_string()).await;

    let btcusd_book_ticker_handle = task::spawn(async move {
        btcusd_bookTicker_client.print_stream().await;
    });


    btcusdt_agg_trade_handle.await.unwrap();
    btcusd_book_ticker_handle.await.unwrap();

}
