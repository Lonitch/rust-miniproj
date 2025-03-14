mod cmdline;
mod completion;
mod shell;
use shell::Shell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shell = Shell::new()?;
    shell.run()
}
