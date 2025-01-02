use mpsc_chat::run_simulation;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    // Run the simulation for 25 seconds
    run_simulation(Duration::from_secs(25)).await;
}
