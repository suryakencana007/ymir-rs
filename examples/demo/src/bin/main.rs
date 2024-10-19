use demo::app::App;

#[tokio::main]
async fn main() -> ymir::Result<()> {
    ymir::startup::start::<App>().await
}
