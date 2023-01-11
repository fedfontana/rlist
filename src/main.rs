#![allow(dead_code, unused)]

use std::{
    fmt::Display,
    fs::{self, File},
    io::Write,
    path::Path,
    str::FromStr,
};

use chrono::Timelike;
use clap::{Parser, Subcommand};
use colored::Colorize;
use dateparser::DateTimeUtc;
use rlist::OrderBy;

use crate::{entry::Entry, rlist::RList};

mod entry;
mod topic;
mod rlist;
mod utils;
mod db;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    #[command(aliases=&["a", "create"])]
    Add {
        name: String,

        url: String,

        #[arg(short, long)]
        author: Option<String>,

        #[arg(short, long, num_args = 1..)]
        topics: Vec<String>,
    },

    #[command(aliases=&["rm", "r", "d", "delete"])]
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

    #[command(aliases=&["ls", "l", "q", "query", "s", "search", "find", "f"])]
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

        #[arg(short, long)]
        sort_by: Option<OrderBy>,

        #[arg(long)]
        desc: bool,

        #[arg(long)]
        from: Option<String>,

        #[arg(long)]
        to: Option<String>,
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
                old_entry.pretty_print_long();
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
        Action::Edit {
            old_name,
            new_name,
            author,
            url,
            topics,
            add_topics,
            clear_topics,
            remove_topics,
        } => {
            let new_entry = rlist.edit(
                old_name,
                new_name,
                author,
                url,
                topics,
                add_topics,
                clear_topics,
                remove_topics,
            )?;
            println!("The new entry is:");
            new_entry.pretty_print_long();
            println!();
        }
        Action::List {
            long,
            query,
            topics,
            author,
            url,
            sort_by,
            desc,
            from,
            to,
        } => {
            let opt_from = if let Some(inner) = from {
                Some(inner.parse::<DateTimeUtc>()?)
            } else {
                None
            };

            let opt_to = if let Some(inner) = to {
                Some(inner.parse::<DateTimeUtc>()?)
            } else {
                None
            };

            let entries = rlist.query(
                query,
                topics,
                author,
                url,
                sort_by,
                desc,
                opt_from,
                opt_to,
            )?;

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
