use rust_socketio::{
    Event, Payload,
    asynchronous::{Client, ClientBuilder},
};
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pad_id = "spike-target";
    let url = "http://localhost:9001";

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let tx_msg = tx.clone();
    let tx_any = tx.clone();
    let client: Client = ClientBuilder::new(url)
        .on("connect", |_, _| {
            Box::pin(async move {
                eprintln!("[spike] CONNECT event fired");
            })
        })
        .on("message", move |payload, _socket| {
            let tx = tx_msg.clone();
            Box::pin(async move {
                let s = match payload {
                    Payload::Text(v) => format!("text:{}", serde_json::to_string(&v).unwrap()),
                    Payload::Binary(b) => format!("binary({} bytes)", b.len()),
                    _ => "string-or-other".to_string(),
                };
                eprintln!("[spike] message: {s}");
                let _ = tx.send(s);
            })
        })
        .on_any(move |evt, payload, _| {
            let tx = tx_any.clone();
            Box::pin(async move {
                let s = match payload {
                    Payload::Text(v) => {
                        format!("any[{evt:?}]:{}", serde_json::to_string(&v).unwrap())
                    }
                    Payload::Binary(b) => format!("any[{evt:?}]:binary({} bytes)", b.len()),
                    _ => format!("any[{evt:?}]:other"),
                };
                eprintln!("[spike] {s}");
                let _ = tx.send(s);
            })
        })
        .on(Event::Error, |err, _| {
            Box::pin(async move {
                eprintln!("[spike] ERROR: {err:?}");
            })
        })
        .connect()
        .await?;

    eprintln!("[spike] connected, emitting CLIENT_READY");
    let client_ready = serde_json::json!({
        "component": "pad",
        "type": "CLIENT_READY",
        "padId": pad_id,
        "sessionID": null,
        "token": "t.spike-token",
        "protocolVersion": 2,
    });
    // Try emitting as a bare Value (rust_socketio converts via Into<Payload>).
    client.emit("message", client_ready.clone()).await?;
    eprintln!("[spike] CLIENT_READY emitted");

    let received = tokio::time::timeout(Duration::from_secs(10), rx.recv()).await;
    match received {
        Ok(Some(msg)) => {
            println!("SPIKE OK — received: {msg}");
            client.disconnect().await?;
            Ok(())
        }
        _ => anyhow::bail!("no inbound message within 10 s — spike FAILED"),
    }
}
