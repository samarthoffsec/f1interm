use clap::Parser;

#[derive(Parser)]
#[command(name = "ferro-f1")]
pub struct Cli {
    #[arg(long, short = 'l')]
    /// Attach to live session
    pub live: bool,

    #[arg(long, short = 'm')]
    /// Run mock generator
    pub mock: bool,

    #[arg(long, short = 'q')]
    /// Quit after demo
    pub quit: bool,

    #[arg(long, short = 'n')]
    /// Disable truecolor output
    pub no_color: bool,

    #[arg(long, short = 'a')]
    /// ASCII mode: replace Braille/box-drawing with ASCII chars
    pub ascii_mode: bool,
}