//! Defines the protocols for all forms of communication.

use crate::{common::log::LoglevelFilter, host::configuration::PluginConfiguration};
use serde::{Deserialize, Serialize};

mod arb_cmd;
pub use arb_cmd::ArbCmd;

mod arb_data;
pub use arb_data::ArbData;

/// Simulator to plugin requests.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Handshake the configuration for reference.
    Configuration(Box<PluginConfiguration>),
    /// Request to initialize the plugin.
    ///
    /// When requested, the plugin should connect to provided downstream and
    /// upstream plugin.
    Init(InitializeRequest),
    /// Request to abort the simulation and stop the plugin.
    Abort,
}

/// Plugin to simulator responses.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Initialization response.
    Init(InitializeResponse),
    /// Success response.
    Success,
}

/// Initialization request.
#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeRequest {
    /// Downstream plugin to connect to.
    pub downstream: Option<String>,
    /// Arbitrary commmands.
    pub arb_cmds: Vec<ArbCmd>,
    /// Prefix for logging.
    pub prefix: String,
    /// LoglevelFilter for logging.
    pub level: LoglevelFilter,
}

/// Initialization response.
#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeResponse {
    // Upstream endpoint.
    pub upstream: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GateStream {
    Hello(String),
    Bye(String),
}
