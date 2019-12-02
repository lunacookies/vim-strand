# vim-strand

A barebones plugin manager for Vim in Rust that takes advantage of Vim’s packages feature. Its (very ambitious) goal is to provide the fastest out-of-the-box fresh plugin installation of all Vim plugin managers.

#### Philosophy

To keep the plugin manager as simple as possible, it only provides one function: re-installing the entire plugin directory each time. This avoids the need for a `clean` command and an `update` command. For maximum speed, strand is written in Rust, using the wonderful [async-std](https://github.com/async-rs/async-std) library for concurrent task support. Additionally, instead of cloning Git repositories by either shelling out to `git` or using a Git binding, strand essentially acts as a parallel `tar.gz` downloader, making use of GitHub’s automated compressed archive generation to avoid downloading extraneous Git info. (This can also be partially achieved with `git clone --depth=1`, but this AFAIK is not compressed like `tar.gz` is.)

#### Motivation

Once I realised that I barely utilised the more advanced features of Vim plugin managers like [vim-plug](https://github.com/junegunn/vim-plug), I decided to start developing a small script to maintain my Vim plugin collection. Conveniently, Vim had just recently gotten support for Pathogen-like `runtimepath` management (`:help packages`), meaning that plugin managers now had only one job – downloading and updating plugins. So far the only plugin manager I’ve seen that takes advantage of packages is [minpac](https://github.com/k-takata/minpac). At one point that duct taped-together script from earlier would download plugins asynchronously using Bash’s job control (`&` and `wait`), leading to very fast install times. To keep things simple, the script just had a hard-coded list of plugins in an array that it would re-download fully each time, instead of keeping track of which plugins still needed to be installed or which plugins needed updating. I decided to rewrite the script in Rust to learn about its async IO capabilities and get better at the language.