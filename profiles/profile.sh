#!/usr/bin/env bash

# cd into the directory the script is being run from
cd "$(dirname $0)"

tmp_dir="$(mktemp -d)"

# Back up both the Vim config and the strand config into a temporary folder if
# they exist.
strand_config="$(strand --config-location)"
orig_strand_config="$tmp_dir/strand_config"
vim_config="$HOME/.vim"
orig_vim_config="$tmp_dir/vim"

[ -e "$strand_config" ] && mv "$strand_config" "$orig_strand_config"
[ -e "$vim_config" ] && mv "$vim_config" "$orig_vim_config"

mkdir "$vim_config"

# Create a strand config with a random set of plugins, one of which is specified
# for vim-plug in run_profile.vim.
echo "
---
plugin_dir: ~/.vim/pack/strand/start

plugins:
  - Git: github@PeterRincker/vim-searchlight
  - Git: github@cakebaker/scss-syntax.vim
  - Git: github@cespare/vim-toml
  - Git: github@christoomey/vim-tmux-navigator
  - Git: github@cocopon/inspecthi.vim
  - Git: github@hail2u/vim-css3-syntax
  - Git: github@jonathanfilip/vim-lucius
  - Git: github@junegunn/fzf.vim
  - Git: github@junegunn/vim-easy-align
  - Git: github@justinmk/vim-dirvish
  - Git: github@kana/vim-textobj-user
  - Git: github@kh3phr3n/python-syntax
  - Git: github@lifepillar/vim-colortemplate
  - Git: github@lifepillar/vim-mucomplete
  - Git: github@othree/html5.vim
  - Git: github@pangloss/vim-javascript
  - Git: github@reedes/vim-textobj-quote
  - Git: github@romainl/vim-cool
  - Git: github@romainl/vim-qf
  - Git: github@rust-lang/rust.vim
  - Git: github@sgur/vim-editorconfig
  - Git: github@tmsvg/pear-tree
  - Git: github@tmux-plugins/vim-tmux
  - Git: github@tpope/vim-commentary
  - Git: github@tpope/vim-endwise
  - Git: github@tpope/vim-fugitive
  - Git: github@tpope/vim-git
  - Git: github@tpope/vim-markdown
  - Git: github@tpope/vim-repeat
  - Git: github@tpope/vim-surround
  - Git: github@tpope/vim-unimpaired
  - Git: github@wellle/targets.vim
" > "$strand_config"

# Install vim- provider:plug before running the profil
curl -fLo ~/.vim/autoload/plug.vim --create-dirs \
    https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim

# Run the profile harness without sourcing the user’s vimrc (but still keeping
# plugins and syntax highlighting)
vim -u NORC "+source profile_harness.vim"

# Output the log to the user
cat profile_log

# Restore the original config files
[ -e "$orig_strand_config" ] && mv "$orig_strand_config" "$strand_config"
[ -e "$orig_vim_config" ] && rm -r "$vim_config" && mv "$orig_vim_config" "$vim_config"
