mod leader;
mod log;
mod state;
mod follower;
mod ledger;
mod client_handler;

use leader::Leader;
use crate::follower::Follower;

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn print_usage() {
    eprintln!(
        "usage:\n\
         \x20 node leader   --client-addr <addr> --follower-addr <addr>\n\
         \x20 node follower --client-addr <addr> --leader-client-addr <addr> --leader-follower-addr <addr>\n\
         \n\
         example (run in separate terminals):\n\
         \x20 node leader   --client-addr 127.0.0.1:9000 --follower-addr 127.0.0.1:9001\n\
         \x20 node follower --client-addr 127.0.0.1:9010 --leader-client-addr 127.0.0.1:9000 --leader-follower-addr 127.0.0.1:9001"
    );
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("leader") => {
            let client_addr = arg_value(&args, "--client-addr")
                .unwrap_or_else(|| "127.0.0.1:9000".to_string());

            let follower_addr = arg_value(&args, "--follower-addr")
                .unwrap_or_else(|| "127.0.0.1:9001".to_string());

            Leader::new().run(&client_addr, &follower_addr).await
        }

        Some("follower") => {
            let client_addr = arg_value(&args, "--client-addr")
                .unwrap_or_else(|| "127.0.0.1:9010".to_string());

            let Some(leader_client_addr) =
                arg_value(&args, "--leader-client-addr")
            else {
                print_usage();
                return Ok(());
            };

            let Some(leader_follower_addr) =
                arg_value(&args, "--leader-follower-addr")
            else {
                print_usage();
                return Ok(());
            };

            Follower::new(leader_client_addr)
                .run(&client_addr, &leader_follower_addr)
                .await
        }

        _ => {
            print_usage();
            Ok(())
        }
    }
}