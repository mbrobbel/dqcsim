use crate::{
    common::log::{callback::LogCallback, tee_file::TeeFile, LoglevelFilter},
    host::configuration::{PluginConfiguration, Seed},
};
use serde::{Deserialize, Serialize};

/// The complete configuration for a DQCsim run.
#[derive(Debug, Deserialize, Serialize)]
pub struct SimulatorConfiguration {
    /// The random seed for the simulation.
    pub seed: Seed,

    /// The verbosity for logging messages to stderr.
    pub stderr_level: LoglevelFilter,

    /// Logs messages to the specified file in addition to stderr. level sets
    /// the minimum importance for a message to be logged to this file.
    pub tee_files: Vec<TeeFile>,

    /// Optional log callback function.
    #[serde(skip)]
    pub log_callback: Option<LogCallback>,

    /// The verbosity for DQCsim itself.
    pub dqcsim_level: LoglevelFilter,

    /// The plugin configurations, from front to back.
    pub plugins: Vec<PluginConfiguration>,
}

impl Default for SimulatorConfiguration {
    /// Generates a default configuration.
    ///
    /// Note that the plugins vector still needs to be populated with at least
    /// two plugins.
    fn default() -> SimulatorConfiguration {
        SimulatorConfiguration {
            seed: Seed::default(),
            stderr_level: LoglevelFilter::Info,
            tee_files: vec![],
            log_callback: None,
            dqcsim_level: LoglevelFilter::Info,
            plugins: vec![],
        }
    }
}
