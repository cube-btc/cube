use crate::communicative::peer::peer::PEER;

/// Returns the connection status with Engine.
pub async fn conn_command(engine: &PEER) {
    let _engine = engine.lock().await;

    match _engine.connection() {
        Some(_) => {
            let addr: String = _engine.addr();
            println!("Alive: {}", addr);
        }
        None => {
            println!("Dead.")
        }
    }
}
