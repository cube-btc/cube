use crate::communicative::peer::peer::PEER;
use crate::communicative::tcp::client::TCPClient;

/// Pings the Engine.
pub async fn ping_command(engine: &PEER) {
    match engine.ping().await {
        Ok(duration) => println!("{} ms", duration.as_millis()),
        Err(_) => println!("Error pinging."),
    }
}
