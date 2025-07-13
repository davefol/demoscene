#![feature(more_float_constants)]
#![feature(iter_array_chunks)]
use clap::{CommandFactory, Parser, Subcommand};

mod bare_window;
mod box_blur_2d;
mod egui_inside;
mod egui_renderer;
mod gpu_context;
mod icosahedron;
mod icosphere;
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
    /// Display a rotating icosphere
    Icosphere(icosphere::Opts),
    /// Blur an image
    #[command(name = "box-blur-2d")]
    BoxBlur2D(box_blur_2d::Opts),
    /// Show egui inside of winit + wgpu
    #[command(name = "egui-inside")]
    EguiInside(egui_inside::Opts),
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
        Some(Demo::Icosphere(opts)) => {
            icosphere::demo(opts)?;
        }
        Some(Demo::BoxBlur2D(opts)) => {
            box_blur_2d::demo(opts)?;
        }
        Some(Demo::EguiInside(_)) => {
            egui_inside::demo()?;
        }
        None => {
            let mut cmd = CLIOptions::command();
            cmd.print_help()?;
        }
    }
    Ok(())
}
