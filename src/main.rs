#[tokio::main]
async fn main() -> muon::error::Result<()> {
    muon::app::run().await
}
