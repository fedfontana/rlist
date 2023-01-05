use clap::{Parser, Subcommand};

mod entry;

#[derive(Parser, Debug)]
struct Args {
   #[command(subcommand)]
   action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    #[command(aliases=&["a"])]
    Add,

    #[command(aliases=&["rm", "r"])]
    Remove,

    #[command(aliases=&["e"])]
    Edit,

    #[command(aliases=&["ls", "l"])]
    List
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("{:?}", args);

    Ok(())
}
