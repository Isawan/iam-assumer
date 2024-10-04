use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use clap::Parser;
use clap_complete::Shell;

const DEFAULT_SOCKET: SocketAddr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0);

#[derive(Parser, Debug, Clone)]
pub enum Args {
    Run(RunArgs),
    GenerateCompletion(GenerateCompletionArgs),
}

#[derive(clap::Args, Debug, Clone)]
pub struct RunArgs {
    /// Socket to listen on
    ///
    /// The host and port to bind the HTTP service. By default a random port is selected
    #[arg(long, default_value_t = DEFAULT_SOCKET, env = "ASSUMER_HTTP_LISTEN")]
    pub(crate) http_listen: SocketAddr,

    /// STS endpoint
    #[arg(long, env = "ASSUMER_STS_ENDPOINT")]
    pub(crate) sts_endpoint: Option<url::Url>,

    /// Role arn
    #[arg(long, env = "ASSUMER_ROLE_ARN")]
    pub(crate) role_arn: String,

    /// Role session name
    #[arg(long, env = "ASSUMER_ROLE_SESSION_NAME")]
    pub(crate) role_session_name: String,

    /// Auth token
    #[arg(long, env = "ASSUMER_AUTH_TOKEN")]
    pub(crate) auth_token: Option<String>,

    /// Command to run
    pub(crate) command: String,

    /// Command arguments
    pub(crate) args: Vec<String>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct GenerateCompletionArgs {
    #[arg(long, env = "ASSUMER_SHELL")]
    pub(crate) shell: Shell,
}
