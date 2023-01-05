# rlist

Reading list manager for the terminal.

## Commands

This command will add that article to the reading list with author fasterthanlime and date added today.
```bash
rlist add https://fasterthanli.me/articles/becoming-fasterthanlime-full-time --author fasterthanlime
```

It is possible to add topics about the articles with the `--topics` option

You can edit entries with `rlist edit identifier`

You can give an identifier to the entry with the `--identifier` or `-i` flag

If you don't provide an identifier, `rlist` will create a random one (a hash of the url+date?)

Either way the program will notify of the creation of the entry by displaying the 

You can delete with `rlist rm ??`

Rename with `rlist mv old_ident new_ident`

List the reading list with `rlist ls`

If you want to ls with a specific filter you can write `rlist ls author:fasterthanlime` or `rlist ls date:today` `rlist ls date:yesterday` `rlist ls date:27-3-22`

Maybe `rlist ls -l`

Maybe `.rlisrc` to save default configs like
- file path
- defaults to long list?
But then i'll need a `--config` if ppl want to change default position of config file

You can choose to use a specific `rlist` db file with the `-f` or `--reading-list-file` option:
```bash
rlist -f ~/my/custom/path.???
```
What format to use for the file?

Integration with ripgrep when using stuff like `rlist query`?


Shorthands like `rlist q` instead of `rlist query`

## Stuff useful for me
Which format to use to save the list? JSON? a sqlite db? yaml?
if using a db, then will need to expose `--export` and `--import` options