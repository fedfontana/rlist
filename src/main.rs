use clap::{Parser, Subcommand};
use dateparser::DateTimeUtc;
use rlist::OrderBy;

use crate::{entry::Entry, rlist::RList};

mod db;
mod entry;
mod rlist;
mod topic;
mod utils;

/// Reading list manager for the command line
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    action: Action,
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

            rlist.add(e)?;
            println!("Entry added to rlist");
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
                println!("Remove these entries:");
                old_entries.iter().for_each(|e| {
                    e.pretty_print(false);
                    println!();
                });
                if old_entries.len() > 1 {
                    println!("Removed a total of {} entries", old_entries.len());
                }
            } else {
                // If neither name or topics is passed to the cli, return err
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
        }
    }
    Ok(())
}
