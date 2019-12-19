<h1 align="center">vim-strand</h1>
<p align="center">
    <img src="https://raw.githubusercontent.com/arzg/resources/master/strand-demo.gif">
</p>

<p align="center"><em>strand installing thirty-one plugins concurrently.</em></p>

[![Actions Status](https://github.com/arzg/vim-strand/workflows/CI/badge.svg)](https://github.com/arzg/vim-strand/actions)

A barebones plugin manager for Vim in Rust that takes advantage of Vim’s packages feature. Its (very ambitious) goal is to provide the fastest out-of-the-box fresh plugin installation of all Vim plugin managers.

#### Usage

There is no CD set up (yet), so to get strand you will have to clone and compile it yourself. This means you will need Rust installed on your system. If you don’t have it already I recommend using [rustup](https://rustup.rs). Once you have Rust installed, run the following commands from inside your clone of this repo:

```bash
> git checkout $(git describe --tags $(git rev-list --tags --max-count 1))
> RUSTFLAGS='--codegen target-cpu=native' cargo install --force --path .
```

This first checks out the last tag (stable version) of the repository, and then compiles it with optimisations specific to your native CPU architecture, finally installing the generated binary to `~/.cargo/bin/strand` for your personal use.

Now all that is left to do is to set up a configuration file – strand uses the YAML format. Put it in the location specified by `strand --config-location`. Here is an example:

```yaml
---
plugin_dir: ~/.vim/pack/strand/start

plugins:
  # GitHub, GitLab and Bitbucket repos are all fully supported
  - Git: github@tpope/vim-surround
  - Git: gitlab@YaBoiBurner/vim-quantum
  - Git: bitbucket@vim-plugins-mirror/vim-surround

  # GitHub is the default Git provider, so ‘github@’ can be elided:
  - Git: tpope/vim-endwise

  - Git: gitlab@YaBoiBurner/vim-quantum:new-styles # Specify a branch name,
  - Git: tpope/vim-unimpaired:v2.0                 # a tag name,
  - Git: romainl/vim-qf:4a97465                    # or a commit hash.

  # Or just the URL of a tar.gz archive
  - Archive: https://codeload.github.com/romainl/vim-qlist/tar.gz/master
```

When you run `strand` in your shell, the specified `plugin_dir` is completely emptied, after which all the plugins in the config file are installed afresh. This property allows you to run `strand` when you want to update your plugins or when you have removed a plugin from your config file and want it gone – all from one command.

The same syntax for specifying plugins also applies to the `install` subcommand, to which you can provide a list of plugins to temporarily install:

```bash
> strand install github@romainl/vim-qf:4a97465 https://codeload.github.com/romainl/vim-qlist/tar.gz/master
```

The next time you run `strand` these plugins will be removed (unless they are in your config file).

#### Philosophy

To keep the plugin manager as simple as possible, it only provides one function: re-installing the entire plugin directory each time. This avoids the need for a `clean` command and an `update` command. For maximum speed, strand is written in Rust, using the wonderful [async-std](https://github.com/async-rs/async-std) library for concurrent task support. Additionally, instead of cloning Git repositories by either shelling out to `git` or using a Git binding, strand essentially acts as a parallel `tar.gz` downloader, making use of the automated compressed archive generation of Git hosting providers like GitHub and Bitbucket to avoid downloading extraneous Git info. (This can also be partially achieved with `git clone --depth=1`, but this AFAIK is not compressed like `tar.gz` is.)

#### Motivation

Once I realised that I barely utilised the more advanced features of Vim plugin managers like [vim-plug](https://github.com/junegunn/vim-plug), I decided to start developing [a small script](https://gist.github.com/arzg/64fcf8601b97e084ec5681c97f292b1a) to maintain my Vim plugin collection. Conveniently, Vim had just recently gotten support for Pathogen-like `runtimepath` management (`:help packages`), meaning that plugin managers now had only one job – downloading and updating plugins. So far the only plugin manager I’ve seen that takes advantage of packages is [minpac](https://github.com/k-takata/minpac). At one point that duct taped-together script from earlier would download plugins asynchronously using Bash’s job control (`&` and `wait`), leading to very fast install times. To keep things simple, the script just had a hard-coded list of plugins in an array that it would re-download fully each time, instead of keeping track of which plugins still needed to be installed or which plugins needed updating. I decided to rewrite the script in Rust to learn about its async IO capabilities and get better at the language.

#### Prior art

- [Pack](https://github.com/maralla/pack) 
- [vim-plug](https://github.com/junegunn/vim-plug)
- [minpac](https://github.com/k-takata/minpac)
