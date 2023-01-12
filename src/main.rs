use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::{Parser, Subcommand};
use dateparser::DateTimeUtc;
use rlist::OrderBy;

use crate::{entry::Entry, rlist::RList};

mod db;
mod entry;
mod rlist;
mod topic;
mod utils;
mod config;

/// Reading list manager for the command line
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    action: Action,

    #[arg(long)]
    db_file: Option<PathBuf>,

    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Add an entry to the reading list
    #[command(aliases=&["a", "create"])]
    Add {
        /// The name of the entry
        name: String,

        /// The content of the entry
        url: String,

        /// The author of the content
        #[arg(short, long)]
        author: Option<String>,

        /// Topics related to the content of the entry
        #[arg(short, long, num_args = 1..)]
        topics: Vec<String>,
    },

    /// Remove an entry from the reading list
    #[command(aliases=&["rm", "r", "d", "delete"])]
    Remove {
        /// The name of the entry you want to remove
        /// Takes precedence over --topics/-t
        name: Option<String>,

        /// Remove ALL of the entries that are linked to ALL of the topics specified after this option
        #[arg(short, long, num_args = 1..)]
        topics: Option<Vec<String>>,
    },

    /// Edit an entry
    #[command(aliases=&["e", "mv"])]
    Edit {
        /// The name of the entry you want to edit
        old_name: String,

        /// The new name of the entry
        new_name: Option<String>,

        /// The new author of the entry
        #[arg(short, long)]
        author: Option<String>,

        /// The new url of the entry
        #[arg(long)]
        url: Option<String>,

        /// Sets the topics of the entry to this list.
        /// Takes precedence over `--add-topics`. `--topics a b c` is the same as `--clear-topics --add-topics a b c`
        #[arg(short, long, num_args = 1..)]
        topics: Option<Vec<String>>,

        /// The topics you want to add to the entry
        #[arg(long, num_args = 1..)]
        add_topics: Option<Vec<String>>,

        /// If set, remove the entry from all of the topics
        #[arg(long)]
        clear_topics: bool,

        /// The list of topics you want the entry to be removed from
        #[arg(long, num_args = 1..)]
        remove_topics: Option<Vec<String>>,
    },

    /// Show the content of your reading list
    #[command(aliases=&["ls", "l", "q", "query", "s", "search", "find", "f"])]
    List {
        /// A substring that the name of the entries must contain
        query: Option<String>,

        /// If set, the result will also show the `added` date and the topics for each entry
        #[arg(short, long)]
        long: bool,

        /// Only show topics that are in all of the topics specified in this option
        #[arg(short, long, num_args = 1..)]
        topics: Option<Vec<String>>,

        /// If set, the list will contain all of the entries that are in at least one of the topics specified with `--topics`
        #[arg(long)]
        or: bool,

        /// Only show the entries that have an author name that contains this substring
        #[arg(short, long)]
        author: Option<String>,

        /// Only show the entries that have urls that contain this substring
        #[arg(long)]
        url: Option<String>,

        /// The attribute used to sort the entries. Options are: name, author, url, added
        #[arg(short, long)]
        sort_by: Option<OrderBy>,

        /// Whether to sort in ascending or descending order. Aliases: `--descending`
        #[arg(short, long, aliases=&["descending"])]
        desc: bool,

        /// Only show entries added after the datetime passed to this option
        #[arg(long)]
        from: Option<String>,

        /// Only show entries added before the datetime passed to this option
        #[arg(long)]
        to: Option<String>,
    },

    /// Imports a set of entries from a yml file
    /// Note that entries with the same name or url as an entry in your reading list will not be imported (and the topics in the import file will not be appended to existing entry)
    Import { path: PathBuf },

    /// Exports the contennt of the whole reading list into a yml file
    Export { path: PathBuf },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let rlist = RList::init(args.db_file)?;

    match args.action {
        Action::Add {
            name,
            author,
            url,
            topics,
        } => {
            let entry = rlist.add(name, url, author, topics)?;
            println!("Entry added to rlist:");
            entry.pretty_print(true);
        }
        Action::Remove { name, topics } => {
            if name.is_some() {
                let old_entry = rlist.remove_by_name(name.unwrap())?;
                print!("Removed entry: \n");
                old_entry.pretty_print(true);
                println!();
            } else if topics.is_some() {
                let old_entries = rlist.remove_by_topics(topics.unwrap())?;
                if old_entries.len() == 0 {
                    println!("No entries were removed");
                    return Ok(());
                }
                println!("Removed these entries:");
                old_entries.iter().for_each(|e| {
                    e.pretty_print(true);
                    println!();
                });
                if old_entries.len() > 1 {
                    println!("Removed a total of {} entries", old_entries.len());
                }
            } else {
                // If neither name or topics is passed to the cli
                return Err(anyhow::anyhow!("No criteria for deletion was selected"));
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
            println!("Here's the edited entry:");
            new_entry.pretty_print(true);
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
            or,
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
                query, topics, author, url, sort_by, desc, opt_from, opt_to, or,
            )?;

            entries.iter().for_each(|e| {
                e.pretty_print(long);
                println!();
            });

            if entries.len() > 0 {
                println!("A total of {} {} matched your query", entries.len(), if entries.len() == 1 { "entry" } else { "entries" });
            }
        }
        Action::Import { path } => {
            let content =
                fs::read_to_string(&path).context("Could not import reading list from file")?;
            let entries: Vec<Entry> = serde_yaml::from_str(&content)
                .context("Could not import reading list from file")?;
            let imported_count = rlist.import(entries)?;

            println!(
                "Imported {imported_count} {word}{source}",
                word = if imported_count == 1 {
                    "entry"
                } else {
                    "entries"
                },
                source = path
                    .to_str()
                    .map(|p| format!(" from {p}"))
                    .unwrap_or_default()
            );
        }
        Action::Export { path } => {
            let entries = rlist.dump_all()?;
            fs::create_dir_all(
                Path::new(&path)
                    .parent()
                    .ok_or(anyhow::anyhow!("Could not create the export file"))?,
            )?;
            let content = serde_yaml::to_string(&entries)
                .context("Could not export the content of your reading list")?;
            fs::write(&path, content)
                .context("Could not export the content of your reading list")?;

            println!(
                "Exported {count} {word}{destination}",
                count = entries.len(),
                word = if entries.len() == 1 {
                    "entry"
                } else {
                    "entries"
                },
                destination = path
                    .to_str()
                    .map(|p| format!(" to {p}"))
                    .unwrap_or_default()
            );
        }
    }
    Ok(())
}
