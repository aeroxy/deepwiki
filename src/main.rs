use deepwiki::{daemon, run};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("__daemon__") {
        let result = daemon::run_daemon().await;
        if let Err(e) = result {
            eprintln!("daemon error: {e:#}");
            std::process::exit(1);
        }
        return;
    }

    if let Err(e) = run().await {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}