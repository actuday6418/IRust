# IRust
Cross Platform Rust Repl

## Keywords / Tips & Tricks

**:reset** => reset repl

**:show** => show repl current code

**:add** *<dep_list>* => add dependencies (requires [cargo-edit](https://github.com/killercup/cargo-edit))

**:load** => load a rust script into the repl

**::** => run a shell command, example `::ls`

You can use arrow keys to cycle through commands history

## Keybindings

**ctrl-l** clear screen
**ctrl-c** exit

<img src="./irust.png" width="80%" height="60%">

## Changeslog

**0.1.5**
- add keybindings `ctrl-c` `ctr-l`
- Fix history regression

**0.1.4**
- Handle parsing errors and output useful info
- Fix add dep regression

**0.1.3**
- Rely on a custom cursor struct to avoid a lot of headaches

**0.1.2**
- load scripts that contains main fn

**0.1.1**
- add **::** to execute shell cmds
- bugfixes
