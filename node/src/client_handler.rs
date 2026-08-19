use common::wire;
use tokio::net::TcpStream;

use crate::client::ClientRequest;
use crate::client::ClientResponse;

pub trait RequestHandler: Send + Sync + 'static {
    async fn handle_request(&self, request: ClientRequest) -> ClientResponse;
}

pub async fn serve_client<H: RequestHandler>(
    mut socket: TcpStream,
    handler: H,
) -> std::io::Result<()> {
    loop {
        let request: ClientRequest = wire::read_message(&mut socket).await?;
        let response = handler.handle_request(request).await;
        wire::write_message(&mut socket, response).await?
    }
}
