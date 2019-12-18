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
echo '
---
plugin_dir: ~/.vim/pack/strand/start

plugins:
  - Git: PeterRincker/vim-searchlight
  - Git: cakebaker/scss-syntax.vim
  - Git: cespare/vim-toml
  - Git: christoomey/vim-tmux-navigator
  - Git: cocopon/inspecthi.vim
  - Git: hail2u/vim-css3-syntax
  - Git: jonathanfilip/vim-lucius
  - Git: junegunn/fzf.vim
  - Git: junegunn/vim-easy-align
  - Git: justinmk/vim-dirvish
  - Git: kana/vim-textobj-user
  - Git: kh3phr3n/python-syntax
  - Git: lifepillar/vim-colortemplate
  - Git: lifepillar/vim-mucomplete
  - Git: othree/html5.vim
  - Git: pangloss/vim-javascript
  - Git: reedes/vim-textobj-quote
  - Git: romainl/vim-cool
  - Git: romainl/vim-qf
  - Git: rust-lang/rust.vim
  - Git: sgur/vim-editorconfig
  - Git: tmsvg/pear-tree
  - Git: tmux-plugins/vim-tmux
  - Git: tpope/vim-commentary
  - Git: tpope/vim-endwise
  - Git: tpope/vim-fugitive
  - Git: tpope/vim-git
  - Git: tpope/vim-markdown
  - Git: tpope/vim-repeat
  - Git: tpope/vim-surround
  - Git: tpope/vim-unimpaired
  - Git: wellle/targets.vim
' > "$strand_config"

# Install vim-plug before running the profile
curl -fLo ~/.vim/autoload/plug.vim --create-dirs \
    https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim

# Run the profile harness without sourcing the user’s vimrc
vim -Nu NORC '+source profile_harness.vim'

# Output the log to the user
cat profile_log

# Restore the original config files
[ -e "$orig_strand_config" ] && mv "$orig_strand_config" "$strand_config"
[ -e "$orig_vim_config" ] && rm -rf "$vim_config" && mv "$orig_vim_config" "$vim_config"
