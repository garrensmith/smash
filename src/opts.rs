use clap::Parser;

/// Smash - A query engine concurrent simulator
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Opts {
    // simulation to run
    #[clap(short, long, default_value = "simple")]
    pub name: String,

    // number of connections at a time
    #[clap(short, long, default_value_t = 10)]
    pub concurrency: u32,

    /// Number of times to run it
    #[clap(short, long, default_value_t = 1)]
    pub iterations: u32,

    /// Wait time for a connection in milliseconds
    #[clap(short, long, default_value_t = 5000)]
    pub wait: u32,

    /// Timeout in milliseconds for a transaction
    #[clap(short, long, default_value_t = 1000)]
    pub timeout: u32,
}
