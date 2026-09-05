mod client_handler;
mod follower;
mod leader;
mod ledger;
mod log;
mod state;
mod storage;

use crate::follower::Follower;
use leader::Leader;

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("leader") => {
            let data_dir =
                arg_value(&args, "--data-dir").unwrap_or_else(|| "data/leader".to_string());

            let client_addr =
                arg_value(&args, "--client-addr").unwrap_or_else(|| "127.0.0.1:9000".to_string());

            let follower_addr =
                arg_value(&args, "--follower-addr").unwrap_or_else(|| "127.0.0.1:9001".to_string());

            Leader::open(&data_dir)?
                .run(&client_addr, &follower_addr)
                .await
        }

        Some("follower") => {
            let data_dir = arg_value(&args, "--data-dir").unwrap_or_else(|| "data/follower".to_string());
            let client_addr = arg_value(&args, "--client-addr").unwrap_or_else(|| "127.0.0.1:9010".to_string());

            let leader_client_addr = arg_value(&args, "--leader-client-addr").unwrap_or_else(|| "127.0.0.1:9000".to_string());

            let leader_follower_addr = arg_value(&args, "--leader-follower-addr").unwrap_or_else(|| "127.0.0.1:9001".to_string());

            Follower::open(&data_dir, leader_client_addr)?
                .run(&client_addr, &leader_follower_addr)
                .await
        }

        _ => {
            eprintln!("usage: node <leader|follower>");
            Ok(())
        }
    }
}
