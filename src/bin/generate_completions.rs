use clap::CommandFactory;
use nginxfmt::cli::Cli;

fn main() {
    nginxfmt::cli::write_packaged_completions().expect("generate completions");
    // This is needed so clap parses metadata for man page generation workflows.
    let _ = Cli::command();
    eprintln!("completions written to packaging/completions/");
}
