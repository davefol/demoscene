use gumdrop::Options;

mod demo_window;

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
}

#[derive(Options)]
struct WindowOpts {}


fn main() -> anyhow::Result<()>{
    let opts = CLIOptions::parse_args_default_or_exit();
    match opts.demo {
        Some(Demo::Window(_)) => {
            demo_window::demo()?;
        }
        None => {}
    }
    Ok(())
}
