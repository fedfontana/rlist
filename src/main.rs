use std::{fs, path::Path};

use anyhow::Context;
use clap::Parser;
use config::Config;
use dateparser::DateTimeUtc;

use crate::{entry::Entry, rlist::RList};

use cli::Action;

mod cli;
mod config;
mod db;
mod entry;
mod rlist;
mod topic;
mod utils;

// TODO: delete time with `recover` subcommand
// - flags for export to also export the deleted ones
// - `cleanup` to hard-delete soft-deleted entries?
// TODO: read time with `read` subcommand
// - `--include-read` for query to include read entries in `search` output
// - `--read` flag on add to add an entry that has already been read
// TODO: `notes` field
// TODO: `adopt` subcommand or similar that takes an URL, finds out its title, sets the base url as
// author, and adds some pre-determined text to notes?

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    let mut config = Config::new_from_arg(args.config)?;
    if let Some(p) = args.db_file {
        config.db_file = p;
    }
    let mut rlist = RList::init(config)?;

    match args.action {
        Action::Add {
            name,
            author,
            url,
            topics,
        } => {
            let mut entry = Entry::new(name, url, author);
            topics.iter().for_each(|t| entry.add_topic(t));

            rlist.db.save(&mut entry)?;
            println!("Entry added to rlist:");
            entry.pretty_print(true, rlist.config.datetime_format)?;
        }
        Action::Remove { name, topics } => {
            match (name, topics) {
                (Some(name), _) => {
                    let old_entry = rlist.remove_by_name(name)?;
                    print!("Removed entry: \n");
                    old_entry.pretty_print(true, rlist.config.datetime_format)?;
                    println!();
                }
                (None, Some(topics)) => {
                    let old_entries = rlist.remove_by_topics(topics)?;
                    if old_entries.len() == 0 {
                        println!("No entries were removed");
                        return Ok(());
                    }

                    println!("Removed these entries:");
                    old_entries.iter().for_each(|e| {
                        if let Err(e) = e.pretty_print(true, &rlist.config.datetime_format) {
                            eprintln!("{}", e);
                        }
                        println!();
                    });

                    if old_entries.len() > 1 {
                        println!("Removed a total of {} entries", old_entries.len());
                    }
                }
                (None, None) => {
                    // If neither name or topics is passed to the cli
                    return Err(anyhow::anyhow!("No criteria for deletion was selected"));
                }
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
            new_entry.pretty_print(true, rlist.config.datetime_format)?;
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
            let opt_from = from.map(|i| i.parse::<DateTimeUtc>()).transpose()?;
            let opt_to = to.map(|i| i.parse::<DateTimeUtc>()).transpose()?;

            let entries = rlist.query(
                query, topics, author, url, sort_by, desc, opt_from, opt_to, or,
            )?;

            entries.iter().for_each(|e| {
                if let Err(e) = e.pretty_print(long, &rlist.config.datetime_format) {
                    eprintln!("{}", e);
                }
                println!();
            });

            if entries.len() > 0 {
                println!(
                    "A total of {} {} matched your query",
                    entries.len(),
                    if entries.len() == 1 {
                        "entry"
                    } else {
                        "entries"
                    }
                );
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
