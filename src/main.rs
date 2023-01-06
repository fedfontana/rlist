#![allow(dead_code, unused)]

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use clap::{Parser, Subcommand};

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
        topics: Vec<String>
    },

    #[command(aliases=&["rm", "r"])]
    Remove {
        id: String,
    },

    #[command(aliases=&["e"])]
    Edit,

    #[command(aliases=&["ls", "l"])]
    List,
}

fn main() -> anyhow::Result<()> {
    
    let args = Args::parse();
    let rlist = RList::init()?;

    match args.action {
        Action::Add { name, author, url, topics, .. } => {
            let e = Entry::new(
                name,
                url,
                author,
            );
            if rlist.add(e)? {
                println!("Entry added to rlist");
            } else {
                println!(
                    "Could not add entry to rlist cause an entry with the same id already exists"
                );
            }
        }
        Action::Remove { id }=> {
            // if let Some(..) = rlist.remove_with_id(&id) {
            //     println!("Remoevd entry with id:{id}");
            // } else {
            //     println!("Could not delete entry with id:{id} cause it was not in the rlist");
            // }
        },
        Action::Edit => unimplemented!(),
        Action::List => {},//rlist.content.iter().for_each(|e| {
            // println!(
            //     "Entry with id:{}, url:{}, author: {}, topics: {} and date: {}",
            //     e.id(),
            //     e.url(),
            //     e.author().unwrap_or("None"),
            //     e.topics().join(","),
            //     e.date()
            // );
        //}),
    }
    Ok(())
}
