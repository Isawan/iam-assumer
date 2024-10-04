use clap::Parser;
use iam_assumer::{config::Args, run};
use nix::sys::signal::Signal::SIGTERM;
use tokio::{
    select,
    signal::unix::SignalKind,
    sync::{mpsc, oneshot},
    task,
};
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())
        .expect("Failed to setup SIGTERM handler");
    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())
        .expect("Failed to setup SIGINT handler");
    let mut sigquit =
        tokio::signal::unix::signal(SignalKind::quit()).expect("Failed to setup SIGQUIT handler");
    let (signal_tx, signal_rx) = mpsc::channel(1);
    let (startup_tx, startup_rx) = oneshot::channel();
    let handler = task::spawn(run(args, signal_rx, startup_tx));

    let _handle_signal = task::spawn(async move {
        loop {
            select! {
                _ = sigterm.recv() => {tracing::info!("Received SIGTERM"); signal_tx.send(SIGTERM).await.expect("Failed to notify SIGTERM");},
                _ = sigint.recv() => {tracing::info!("Received SIGINT"); signal_tx.send(SIGTERM).await.expect("Failed to notify SIGTERM");},
                _ = sigquit.recv() => {tracing::info!("Received SIGQUIT"); signal_tx.send(SIGTERM).await.expect("Failed to notify SIGTERM");},
            }
        }
    });

    if (startup_rx.await).is_ok() {
        tracing::info!("Server ready");
    }

    let result = handler.await.unwrap();
    tracing::info!("Server shutdown");
    result
}
