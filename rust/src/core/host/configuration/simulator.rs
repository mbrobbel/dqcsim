use crate::{
    common::{
        error::{inv_arg, Result},
        log::{callback::LogCallback, tee_file::TeeFileConfiguration, LoglevelFilter},
        types::PluginType,
    },
    host::configuration::{PluginConfiguration, ReproductionPathStyle, Seed},
};

/// The complete configuration for a DQCsim run.
#[derive(Debug)]
pub struct SimulatorConfiguration {
    /// The random seed for the simulation.
    pub seed: Seed,

    /// The verbosity for logging messages to stderr.
    pub stderr_level: LoglevelFilter,

    /// Logs messages to the specified file in addition to stderr. level sets
    /// the minimum importance for a message to be logged to this file.
    pub tee_files: Vec<TeeFileConfiguration>,

    /// Optional log callback function.
    pub log_callback: Option<LogCallback>,

    /// The verbosity for DQCsim itself.
    pub dqcsim_level: LoglevelFilter,

    /// The plugin configurations, from front to back.
    pub plugins: Vec<Box<dyn PluginConfiguration>>,

    /// The path style used when writing the reproduction file.
    pub reproduction_path_style: Option<ReproductionPathStyle>,
}

impl SimulatorConfiguration {
    /// Sets the random seed.
    pub fn with_seed(mut self, seed: impl Into<Seed>) -> SimulatorConfiguration {
        self.seed = seed.into();
        self
    }

    /// Sets the stderr loglevel.
    pub fn with_stderr_level(mut self, level: impl Into<LoglevelFilter>) -> SimulatorConfiguration {
        self.stderr_level = level.into();
        self
    }

    /// Adds a tee file.
    pub fn with_tee_file(mut self, tee: impl Into<TeeFileConfiguration>) -> SimulatorConfiguration {
        self.tee_files.push(tee.into());
        self
    }

    /// Sets the log callback.
    pub fn with_log_callback(mut self, callback: impl Into<LogCallback>) -> SimulatorConfiguration {
        self.log_callback = Some(callback.into());
        self
    }

    /// Sets the verbosity for DQCsim itself.
    pub fn with_dqcsim_level(mut self, level: impl Into<LoglevelFilter>) -> SimulatorConfiguration {
        self.dqcsim_level = level.into();
        self
    }

    /// Adds a new plugin to the pipeline.
    pub fn with_plugin(
        mut self,
        plugin_configuration: impl Into<Box<dyn PluginConfiguration>>,
    ) -> SimulatorConfiguration {
        self.plugins.push(plugin_configuration.into());
        self
    }

    /// Sets the reproduction path style.
    pub fn with_reproduction_path_style(
        mut self,
        reproduction_path_style: impl Into<ReproductionPathStyle>,
    ) -> SimulatorConfiguration {
        self.reproduction_path_style = Some(reproduction_path_style.into());
        self
    }

    /// Disables the reproduction logging system.
    pub fn without_reproduction(mut self) -> SimulatorConfiguration {
        self.reproduction_path_style = None;
        self
    }

    /// Disables all logging methods by setting `stderr_level` and
    /// `dqcsim_level` to LoglevelFilter::Off, `tee_files` to an empty vector
    /// and `log_callback` to none.
    pub fn without_logging(mut self) -> SimulatorConfiguration {
        self.stderr_level = LoglevelFilter::Off;
        self.dqcsim_level = LoglevelFilter::Off;
        self.tee_files = vec![];
        self.log_callback = None;
        self
    }

    /// Optimizes the source verbosity levels, such that they are no more
    /// verbose than the most verbose sink.
    pub fn optimize_loglevels(&mut self) {
        // Find the verbosity of the most verbose sink.
        let mut max_dqcsim_verbosity = self.stderr_level;
        for tee in &self.tee_files {
            if tee.filter > max_dqcsim_verbosity {
                max_dqcsim_verbosity = tee.filter;
            }
        }
        if let Some(cb) = self.log_callback.as_ref() {
            if cb.filter > max_dqcsim_verbosity {
                max_dqcsim_verbosity = cb.filter;
            }
        }

        // Clamp the verbosities of the sources.
        if self.dqcsim_level > max_dqcsim_verbosity {
            self.dqcsim_level = max_dqcsim_verbosity;
        }
        for plugin in &mut self.plugins {
            plugin.limit_verbosity(max_dqcsim_verbosity);
        }
    }

    /// Verifies that the plugins are specified correctly.
    ///
    /// This checks that there is exactly one frontend and exactly one backend.
    /// If this is true but they're not in the right place, they are silently
    /// moved. This also ensures that there are no duplicate plugin names, and
    /// auto-names empty plugin names.
    pub fn check_plugin_list(&mut self) -> Result<()> {
        // Check and fix frontend.
        let mut frontend_idx = None;
        for (i, plugin) in self.plugins.iter().enumerate() {
            if let PluginType::Frontend = plugin.get_type() {
                if frontend_idx.is_some() {
                    inv_arg("duplicate frontend")?
                } else {
                    frontend_idx = Some(i);
                }
            }
        }
        match frontend_idx {
            Some(0) => (),
            Some(x) => {
                let plugin = self.plugins.remove(x);
                self.plugins.insert(0, plugin);
            }
            None => inv_arg("missing frontend")?,
        }

        // Check and fix backend.
        let mut backend_idx = None;
        for (i, plugin) in self.plugins.iter().enumerate() {
            if let PluginType::Backend = plugin.get_type() {
                if backend_idx.is_some() {
                    inv_arg("duplicate backend")?
                } else {
                    backend_idx = Some(i);
                }
            }
        }
        match backend_idx {
            Some(x) => {
                if x != self.plugins.len() - 1 {
                    let plugin = self.plugins.remove(x);
                    self.plugins.push(plugin);
                }
            }
            None => inv_arg("missing backend")?,
        }

        // Auto-name plugins and check for conflicts.
        let mut names = std::collections::HashSet::new();
        for (i, plugin) in self.plugins.iter_mut().enumerate() {
            plugin.set_default_name(match plugin.get_type() {
                PluginType::Frontend => "front".to_string(),
                PluginType::Operator => format!("op{}", i),
                PluginType::Backend => "back".to_string(),
            });
            let name = plugin.get_name();
            if !names.insert(name) {
                inv_arg(format!("duplicate plugin name '{}'", plugin.get_name()))?;
            }
        }

        Ok(())
    }
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
            dqcsim_level: LoglevelFilter::Trace,
            plugins: vec![],
            reproduction_path_style: Some(ReproductionPathStyle::Keep),
        }
    }
}
