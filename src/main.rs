#![allow(dead_code, unused)]

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::{entry::Entry, rlist::RList};

mod entry;
mod rlist;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    #[command(aliases=&["a"])]
    Add {
        name: String,

        url: String,

        #[arg(long)]
        id: Option<String>,

        #[arg(short, long)]
        author: Option<String>,

        #[arg(short, long, num_args = 1..)]
        topics: Vec<String>,
    },

    #[command(aliases=&["rm", "r"])]
    Remove { name: String },

    // #[command(aliases=&["e"])]
    // Edit,
    #[command(aliases=&["ls", "l"])]
    List {
        query: Option<String>,

        #[arg(short, long)]
        long: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let rlist = RList::init()?;

    match args.action {
        Action::Add {
            name,
            author,
            url,
            topics,
            ..
        } => {
            let e = Entry::new(name, url, author, topics, None);
            if rlist.add(e)? {
                println!("Entry added to rlist");
            } else {
                println!(
                    "Could not add entry to rlist cause an entry with the same id already exists"
                );
            }
        }
        Action::Remove { name } => {
            let old_entry = rlist.remove_by_name(name)?;
            print!("Removed entry: \n");
            old_entry.pretty_print();
            println!();
        }
        //Action::Edit => unimplemented!(),
        Action::List { long, query } => {
            let entries = if query.is_some() {
                rlist.query(query.unwrap())?
            } else {
                rlist.get_all()?
            };

            if long {
                entries.iter().for_each(|e| {
                    e.pretty_print_long();
                    println!();
                });
            } else {
                entries.iter().for_each(|e| {
                    e.pretty_print();
                    println!();
                });
            }
        }
    }
    Ok(())
}
