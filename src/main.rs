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

    let rlist_file_path = format!(
        "{}/rlist.json",
        dirs::home_dir()
            .ok_or(anyhow::anyhow!("Could not find home directory"))?
            .display()
    );

    let rlist_file_exists = Path::new(&rlist_file_path).exists();

    let list_content = if rlist_file_exists {
        serde_json::from_str(&fs::read_to_string(&rlist_file_path)?)?
    } else {
        Vec::new()
    };

    let mut rlist = RList::new(list_content);

    match args.action {
        Action::Add { id, author, url, topics } => {
            let e = Entry::new(
                id.unwrap_or(1.to_string()),
                url,
                author,
                topics,
                chrono::offset::Utc::now(),
            );
            if rlist.add(e) {
                println!("Entry added to rlist");
            } else {
                println!(
                    "Could not add entry to rlist cause an entry with the same id already exists"
                );
            }
        }
        Action::Remove { id }=> {
            if let Some(..) = rlist.remove_with_id(&id) {
                println!("Remoevd entry with id:{id}");
            } else {
                println!("Could not delete entry with id:{id} cause it was not in the rlist");
            }
        },
        Action::Edit => unimplemented!(),
        Action::List => rlist.content.iter().for_each(|e| {
            println!(
                "Entry with id:{}, url:{}, author: {}, topics: {} and date: {}",
                e.id(),
                e.url(),
                e.author().unwrap_or("None"),
                e.topics().join(","),
                e.date()
            );
        }),
    }

    //println!("{:?}", args);
    let mut file = if !rlist_file_exists {
        File::create(&rlist_file_path)?
    } else {
        File::options().write(true).open(&rlist_file_path)?
    };
    
    println!("{}", serde_json::to_string_pretty(&rlist.content)?);

    let a: &[_] = rlist.content.as_ref();

    file.write_all(serde_json::to_string(a)?.as_bytes())?;

    Ok(())
}
