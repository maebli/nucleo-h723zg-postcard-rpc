use std::{
    io::{stdout, Write},
    time::{Duration, Instant},
};

use software_host::{client::WorkbookClient, icd, read_line};

#[tokio::main]
async fn main() {
    println!("Connecting to USB device...");
    let client = WorkbookClient::new();
    println!("Connected! Pinging 42");
    let ping = client.ping(42).await.unwrap();
    println!("Got: {ping}.");
    let uid = client.get_id().await.unwrap();
    println!("ID: {uid:016X}");
    println!();

    // Begin repl...
    loop {
        print!("> ");
        stdout().flush().unwrap();
        let line = read_line().await;
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["ping"] => {
                let ping = client.ping(42).await.unwrap();
                println!("Got: {ping}.");
            }
            ["ping", n] => {
                let Ok(idx) = n.parse::<u32>() else {
                    println!("Bad u32: '{n}'");
                    continue;
                };
                let ping = client.ping(idx).await.unwrap();
                println!("Got: {ping}.");
            }
            ["rgb", pos, r, g, b] => {
                let (Ok(pos), Ok(r), Ok(g), Ok(b)) = (pos.parse(), r.parse(), g.parse(), b.parse())
                else {
                    panic!();
                };
                client.set_rgb_single(pos, r, g, b).await.unwrap();
            }
            ["rgball", r, g, b] => {
                let (Ok(r), Ok(g), Ok(b)) = (r.parse(), g.parse(), b.parse()) else {
                    panic!();
                };
                client.set_all_rgb_single(r, g, b).await.unwrap();
            }
            ["schema"] => {
                let schema = client.client.get_schema_report().await.unwrap();

                println!();
                println!("# Endpoints");
                println!();
                for ep in &schema.endpoints {
                    println!("* '{}'", ep.path);
                    println!("  * Request:  {}", ep.req_ty);
                    println!("  * Response: {}", ep.resp_ty);
                }

                println!();
                println!("# Topics Client -> Server");
                println!();
                for tp in &schema.topics_in {
                    println!("* '{}'", tp.path);
                    println!("  * Message: {}", tp.ty);
                }

                println!();
                println!("# Topics Client <- Server");
                println!();
                for tp in &schema.topics_out {
                    println!("* '{}'", tp.path);
                    println!("  * Message: {}", tp.ty);
                }
                println!();
            }
            other => {
                println!("Error, didn't understand '{other:?};");
            }
        }
    }
}