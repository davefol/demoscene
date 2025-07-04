use gumdrop::Options;

mod demo_window;
mod demo_single_triangle;
mod gpu_context;

#[derive(Options)]
struct CLIOptions {
    #[options(help = "print help message")]
    help: bool,

    #[options(command)]
    demo: Option<Demo> 
}

#[derive(Options)]
enum Demo {
    Window(WindowOpts),
    SingleTriangle(SingleTriangleOpts),
}

#[derive(Options)]
struct WindowOpts {}

#[derive(Options)]
struct SingleTriangleOpts {}

fn main() -> anyhow::Result<()>{
    let opts = CLIOptions::parse_args_default_or_exit();
    match opts.demo {
        Some(Demo::Window(_)) => {
            demo_window::demo()?;
        }
        Some(Demo::SingleTriangle(_)) => {
            demo_single_triangle::demo()?;
        }
        None => {}
    }
    Ok(())
}
