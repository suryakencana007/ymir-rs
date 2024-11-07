use adaptor::app::App;

#[tokio::main]
async fn main() -> ymir::Result<()> {
    ymir::startup::run::<App>().await
}
