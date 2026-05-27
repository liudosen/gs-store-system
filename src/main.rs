mod app;
mod common;
mod domains;
mod infra;
mod routes;

#[tokio::main]
async fn main() {
    if let Err(error) = app::bootstrap().await {
        eprintln!("server bootstrap failed: {error:#}");
        std::process::exit(1);
    }
}
