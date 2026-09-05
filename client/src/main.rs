use common::{Cents, ClientRequest, ClientResponse, wire};
use tokio::net::TcpStream;
use std::env;

fn print_usage() {
    eprintln!(
        "usage:\n\
         \x20 client [node-addr] deposit  <account> <dollars>\n\
         \x20 client [node-addr] withdraw <account> <dollars>\n\
         \x20 client [node-addr] transfer <from> <to> <dollars>\n\
         \x20 client [node-addr] balance  <account> [--read-after <offset>]\n\
         \n\
         examples:\n\
         \x20 client deposit alice 100.00\n\
         \x20 client transfer alice bob 40.00\n\
         \x20 client balance alice\n\
         \x20 client 127.0.0.1:9010 balance alice --read-after 3"
    );
}

fn parse_dollars(s: &str) -> Result<Cents, String> {
    let (whole, frac) = s.split_once(".").unwrap_or((s, ""));
    let whole: i64 = whole.parse().map_err(|_| format!("not a number {s}"))?;
    let frac_cents: i64 = match frac.len() {
           0 => 0,
           1 => frac.parse::<i64>().map_err(|_| format!("not a number: {s}"))? * 10,
           2 => frac.parse::<i64>().map_err(|_| format!("not a number: {s}"))?,
           _ => return Err("too many decimal places".to_string()),
       };
       Ok(whole * 100 + frac_cents)
}

async fn send(node_addr: &str, request: ClientRequest) -> std::io::Result<ClientResponse> {
    let mut socket = TcpStream::connect(node_addr).await?;
    wire::write_message(&mut socket, &request).await?;
    wire::read_message(&mut socket).await
}

fn format_dollars(cents: Cents) -> String{
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.abs();
    format!("{sign}{}.{:02}", abs / 100, abs % 100)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let raw_args: Vec<String> = env::args().collect();

    if raw_args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    let (node_addr, args) = if raw_args[1].contains(':') {
           (raw_args[1].clone(), raw_args[2..].to_vec())
       } else {
           ("127.0.0.1:9000".to_string(), raw_args[1..].to_vec())
       };
       let command = match args.first().map(String::as_str) {
           Some(cmd) => cmd,
           None => {
               print_usage();
               return Ok(());
           }
       };

       let request = match command {
               "deposit" | "withdraw" => {
                   let (Some(account), Some(amount_str)) = (args.get(1), args.get(2)) else {
                       print_usage();
                       return Ok(());
                   };
                   let amount = match parse_dollars(amount_str) {
                       Ok(a) => a,
                       Err(e) => {
                           eprintln!("{e}");
                           return Ok(());
                       }
                   };
                   if command == "deposit" {
                       ClientRequest::Deposite {
                           account: account.clone(),
                           amount,
                       }
                   } else {
                       ClientRequest::Withdraw {
                           account: account.clone(),
                           amount,
                       }
                   }
               }
               "transfer" => {
                   let (Some(from), Some(to), Some(amount_str)) = (args.get(1), args.get(2), args.get(3)) else {
                       print_usage();
                       return Ok(());
                   };
                   let amount = match parse_dollars(amount_str) {
                       Ok(a) => a,
                       Err(e) => {
                           eprintln!("{e}");
                           return Ok(());
                       }
                   };
                   ClientRequest::Transfer {
                       from: from.clone(),
                       to: to.clone(),
                       amount,
                   }
               }
               "balance" => {
                   let Some(account) = args.get(1) else {
                       print_usage();
                       return Ok(());
                   };
                   let read_after = args
                       .iter()
                       .position(|a| a == "--read-after")
                       .and_then(|i| args.get(i + 1))
                       .and_then(|s| s.parse::<u64>().ok());
                   ClientRequest::GetBalance {
                       account: account.clone(),
                       read_after,
                   }
               }
               _ => {
                   print_usage();
                   return Ok(());
               }
           };
           match send(&node_addr, request).await? {
               ClientResponse::WriteAck { offset } => {
                   println!("ok, committed at offset {offset}");
               }
               ClientResponse::Balance {
                   account,
                   amount,
                   as_of_offset,
               } => {
                   println!(
                       "{account}: ${} (as of offset {as_of_offset})",
                       format_dollars(amount)
                   );
               }
               ClientResponse::NotLeader { leader_addr } => match leader_addr {
                   Some(addr) => println!("not the leader — writes go to {addr}"),
                   None => println!("not the leader"),
               },
               ClientResponse::Error { message } => {
                   println!("error: {message}");
               }
           }
           Ok(())
}
