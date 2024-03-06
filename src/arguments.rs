use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The path to the CIF file or directory containing CIF files.
    #[arg(short, long, group = "execute")]
    pub cif: Option<String>,
    /// Instructions as a string or a path to a file containing instructions.
    #[arg(short, long, requires = "execute")]
    pub instructions: Option<String>,
    /// Print examples of how to use the program.
    #[arg(short, long, group = "example")]
    pub examples: bool,
    /// Show additional debug information.
    #[arg(short, long)]
    pub verbose: bool,
}
