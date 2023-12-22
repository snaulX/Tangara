use tangara_highlevel::Package;
use crate::entrypoint_generator::EntrypointGenerator;

pub struct Config {
    /// Enable generation of internal types and members.
    /// Default value: `false`
    pub enable_internal: bool
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_internal: false
        }
    }
}

pub struct RustGenerator {
    config: Config,
    package: Package
}

impl RustGenerator {
    pub fn new(package: Package, config: Config) -> Self {
        Self {
            config,
            package
        }
    }

    pub fn generate_entrypoint(mut self) -> EntrypointGenerator {
        EntrypointGenerator::new(self.package, self.config)
    }
}