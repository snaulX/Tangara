use tangara_highlevel::Package;
use crate::entrypoint_generator::EntrypointGenerator;
use crate::source_generator::SourceGenerator;

pub struct Config {
    /// Enable generation of internal types and members.
    /// Default value: `false`
    pub enable_internal: bool,
    /// Default name for constructor definition.
    /// Bindings generator for constructors generates function in impl
    /// named like this: `fn {ctor_name}{ctor_index}(args) -> Self`, for example `fn new0(arg: i32) -> Self`.
    /// Default value: `"new"`
    pub ctor_name: String,
    /// Generate implementation of [Default] trait for structs that have constructor with empty arguments.
    /// Default value: `true`
    pub generate_default: bool,
    /// Name of dynamic library's function which loads Tangara data.
    /// Default value: `"tgLoad"`
    pub load_name: String
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_internal: false,
            ctor_name: "new".to_string(),
            generate_default: true,
            load_name: "tgLoad".to_string()
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

    pub fn generate_bindings(mut self) -> SourceGenerator {
        SourceGenerator::new(self.package, self.config)
    }
}