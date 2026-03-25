mod completion;
mod diagnostics;
mod hover;
mod server;
mod span_utils;
mod symbols;

use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(server::PatchLangServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
