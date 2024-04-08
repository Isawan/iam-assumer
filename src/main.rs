use std::sync::mpsc::channel;

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

    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();
    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt()).unwrap();
    let mut sigquit = tokio::signal::unix::signal(SignalKind::quit()).unwrap();
    let (signal_tx, signal_rx) = mpsc::channel(1);
    let (startup_tx, startup_rx) = oneshot::channel();
    let handler = task::spawn(run(args, signal_rx, startup_tx));

    let _handle_signal = task::spawn(async move {
        loop {
            select! {
                _ = sigterm.recv() => {tracing::info!("Received SIGTERM"); signal_tx.send(SIGTERM).await.unwrap();},
                _ = sigint.recv() => {tracing::info!("Received SIGINT"); signal_tx.send(SIGTERM).await.unwrap();},
                _ = sigquit.recv() => {tracing::info!("Received SIGQUIT"); signal_tx.send(SIGTERM).await.unwrap();},
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
