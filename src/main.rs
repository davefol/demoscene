#![feature(more_float_constants)]
#![feature(iter_array_chunks)]
use clap::{CommandFactory, Parser, Subcommand};

mod bare_window;
mod gpu_context;
mod icosahedron;
mod single_triangle;

#[derive(Parser)]
#[command(version)]
struct CLIOptions {
    #[command(subcommand)]
    demo: Option<Demo>,
}

#[derive(Subcommand)]
enum Demo {
    /// Show a window with nothing on it
    BareWindow(bare_window::Opts),
    /// Display a single triangle
    SingleTriangle(single_triangle::Opts),
    /// Display a rotating icosahedron
    Icosahedron(icosahedron::Opts),
}

fn main() -> anyhow::Result<()> {
    let opts = CLIOptions::parse();
    match opts.demo {
        Some(Demo::BareWindow(_)) => {
            bare_window::demo()?;
        }
        Some(Demo::SingleTriangle(_)) => {
            single_triangle::demo()?;
        }
        Some(Demo::Icosahedron(_)) => {
            icosahedron::demo()?;
        }
        None => {
            let mut cmd = CLIOptions::command();
            cmd.print_help()?;
        }
    }
    Ok(())
}
