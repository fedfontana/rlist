# rlist

Reading list manager for the terminal.

## Installation

```console
cargo install rlist
```

## Basic usage

Add entries with
```console
rlist add <name> <title> [-a <author>] [-t <topic1> <topic2> ...]
```

Query your reading list:
```console
rlist ls -l
rlist query <name> # filter results by name
rlist ls --sort-by url --from 2023-01-10
```
If you need to filter the results in other ways, please run `rlist query --help`

Edit entries with
```console
rlist edit <old name> <new name> -a <new author> -t <new topics>
rlist edit <old name> --clear topics
```

Delete entries:
```console
rlist delete <name>
rlist delete -t <topic1> <topic2>
```

For more info run `rlist <subcommand> --help/-h`

If you want to change the rlist database location (default is `$HOME/rlist/rlist.sqlite`), run `rlist --db-file <new path>`, or add 
```yml
db_file: <new path>
```
to your `rlist.yml`, located by default in `$HOME/.config/rlist.yml` (if you want to run rlist with a different config, you can always run `rlist --config <config path> <subcommand>`)