pub(crate) mod app;
pub mod config;
pub(crate) mod http;

use std::net::SocketAddr;

use crate::app::AppState;
use aws_config::BehaviorVersion;
use axum::Router;
use config::{Args, RunArgs};
use hyper::{body::Incoming, Request};
use hyper_util::rt::{TokioExecutor, TokioIo};
use nix::{libc::pid_t, sys::signal::Signal, unistd::Pid};
use rand::{distributions, Rng};
use tokio::{
    net::TcpListener,
    process::Command,
    select,
    sync::{mpsc::Receiver, oneshot::Sender},
};
use tower::Service;
use tracing::error;

#[derive(Debug)]
#[allow(dead_code)]
pub struct StartUpNotify<T> {
    msg: T,
}

async fn serve(listener: TcpListener, app: Router) {
    loop {
        let (socket, _remote_addr) = listener.accept().await.unwrap();

        let tower_service = app.clone();

        // Spawn a task to handle the connection. That way we can multiple connections
        // concurrently.
        tokio::spawn(async move {
            let socket = TokioIo::new(socket);

            // Hyper also has its own `Service` trait and doesn't use tower. We can use
            // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
            // `tower::Service::call`.
            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                // We have to clone `tower_service` because hyper's `Service` uses `&self` whereas
                // tower's `Service` requires `&mut self`.
                //
                // We don't need to call `poll_ready` since `Router` is always ready.
                tower_service.clone().call(request)
            });

            if let Err(err) = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection(socket, hyper_service)
                .await
            {
                tracing::warn!("failed to serve connection: {err:#}");
            }
        });
    }
}

pub async fn run(
    config: Args,
    signals: Receiver<Signal>,
    startup: Sender<StartUpNotify<SocketAddr>>,
) -> Result<(), ()> {
    match config {
        Args::Run(args) => run_cmd(args, signals, startup).await,
    }
}

// refactoring setup code in run_server
pub async fn setup_server(config: &RunArgs) -> Result<aws_sdk_sts::Client, ()> {
    // Set up AWS SDK
    let aws_config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .load()
        .await;
    let mut sts_config = aws_sdk_sts::config::Builder::from(&aws_config);
    if let Some(endpoint) = &config.sts_endpoint {
        sts_config = sts_config.endpoint_url(endpoint.clone());
    }
    let sts = aws_sdk_sts::Client::from_conf(sts_config.build());

    Ok(sts)
}

pub(crate) async fn run_cmd(
    config: RunArgs,
    signals: Receiver<Signal>,
    startup: Sender<StartUpNotify<SocketAddr>>,
) -> Result<(), ()> {
    let sts = setup_server(&config).await.unwrap();

    let bind_addr = config.http_listen;
    let listener = TcpListener::bind(&bind_addr).await.unwrap();
    let local_addr = listener.local_addr().unwrap();

    // random base64 encoded string
    let auth_token: String = config.auth_token.clone().unwrap_or_else(|| {
        rand::thread_rng()
            .sample_iter(distributions::Alphanumeric)
            .take(20)
            .map(char::from)
            .collect()
    });

    let app = app::assumer_app(AppState::new(
        sts,
        auth_token.clone(),
        config.role_arn.clone(),
        config.role_session_name.clone(),
    ));

    let process_handler = tokio::spawn(spawn(
        config.clone(),
        auth_token,
        local_addr.port(),
        signals,
    ));

    startup
        .send(StartUpNotify { msg: local_addr })
        .expect("Sender channel has already been used");

    select! {
        _ = process_handler => {
            tracing::debug!("Child process exited");
        },
        _ = serve(listener, app) => {
            tracing::error!("Server exited unexpectedly");
        }
    }

    tracing::debug!("Shutting down server");
    Ok(())
}

pub(crate) async fn spawn(
    config: RunArgs,
    auth_token: String,
    port: u16,
    mut signals: Receiver<Signal>,
) {
    tracing::trace!("Spawning child process");
    let mut process = Command::new(&config.command)
        .args(&config.args)
        .env("AWS_CONTAINER_AUTHORIZATION_TOKEN", auth_token)
        .env(
            "AWS_CONTAINER_CREDENTIALS_FULL_URI",
            format!("http://localhost:{port}/get-credentials"),
        )
        .env("AWS_SHARED_CREDENTIALS_FILE", "/dev/null")
        .env_remove("AWS_ACCESS_KEY_ID")
        .env_remove("AWS_SECRET_ACCESS_KEY")
        .env_remove("AWS_SESSION_TOKEN")
        .env_remove("AWS_PROFILE")
        .spawn()
        .expect("failed to spawn command");

    loop {
        select! {
            _ = process.wait() => {
                break;
            },
            signal = signals.recv() => match signal {
                Some(s ) => {
                    if let Some(pid) = process.id() {
                        tracing::debug!("Received signal, killing child process");
                        if let Err(e) = nix::sys::signal::kill(Pid::from_raw(pid as pid_t), s){
                            error!(error = ?e, "Failed to kill child process");
                        }
                    }
                }
                None => {
                    panic!("Signal channel closed unexpectedly");
                }
            }
        }
    }
}
