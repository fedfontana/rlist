use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::rlist::OrderBy;

/// Reading list manager for the command line
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,

    /// The path to the database used by rlist to save your reading list. Default path is: `~/rlist/rlist.sqlite` on unix based systems
    /// Note that this argument will take precedence over the option set in the config file, if any.
    #[arg(long)]
    pub db_file: Option<PathBuf>,

    /// The path to the (optional) config file. Default config path (automatically picked up by the program) is `~/.config/rlist.yml` on unix based systems
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Action {
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
