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
    Remove { 
        /// Takes precedence over --topics/-t
        name: Option<String>,

        #[arg(short, long, num_args = 1..)]
        topics: Option<Vec<String>>,
    },

    #[command(aliases=&["e", "mv"])]
    Edit {
        old_name: String,

        new_name: Option<String>,

        #[arg(short, long)]
        author: Option<String>,

        #[arg(long)]
        url: Option<String>,

        /// Takes precedence over --add-topics. `--topics a b c` is the same as `--clear-topics --add-topics a b c`
        #[arg(short, long, num_args = 1..)]
        topics: Option<Vec<String>>,

        #[arg(long, num_args = 1..)]
        add_topics: Option<Vec<String>>,

        #[arg(long)]
        clear_topics: bool,

        #[arg(long, num_args = 1..)]
        remove_topics: Option<Vec<String>>,
    },

    #[command(aliases=&["ls", "l", "q", "query"])]
    List {
        query: Option<String>,

        #[arg(short, long)]
        long: bool,

        #[arg(short, long, num_args = 1..)]
        topics: Option<Vec<String>>,

        #[arg(short, long)]
        author: Option<String>,

        #[arg(long)]
        url: Option<String>,
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
        Action::Remove { name, topics } => {
            if name.is_some() {
                let old_entry = rlist.remove_by_name(name.unwrap())?;
                print!("Removed entry: \n");
                old_entry.pretty_print();
                println!();
            } else if topics.is_some() {
                let old_entries = rlist.remove_by_topics(topics.unwrap())?;
                if old_entries.len() == 0 {
                    println!("No entries were removed");
                    return Ok(());
                }
                println!("Remove these entries:");
                old_entries.iter().for_each(|e| {
                    e.pretty_print();
                    println!();
                });
                if old_entries.len() > 1 {
                    println!("Removed a total of {} entries", old_entries.len());
                }
            } else {
                return Err(anyhow::anyhow!("You gotta select something to delete boi"));
            }
        }
        Action::Edit { old_name, new_name, author, url, topics, add_topics, clear_topics, remove_topics }=> {
            let new_entry = rlist.edit(old_name, new_name, author, url, topics, add_topics, clear_topics, remove_topics)?;
            println!("The new entry is:");
            new_entry.pretty_print_long();
            println!();
        },
        Action::List { long, query, topics, author, url } => {
            let entries = if query.is_some() || topics.is_some() || author.is_some() || url.is_some() {
                rlist.query(query, topics, author, url)?
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
