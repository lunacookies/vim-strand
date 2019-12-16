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
  - provider: GitHub
    user: PeterRincker
    repo: vim-searchlight

  - provider: GitHub
    user: cakebaker
    repo: scss-syntax.vim

  - provider: GitHub
    user: cespare
    repo: vim-toml

  - provider: GitHub
    user: christoomey
    repo: vim-tmux-navigator

  - provider: GitHub
    user: cocopon
    repo: inspecthi.vim

  - provider: GitHub
    user: hail2u
    repo: vim-css3-syntax

  - provider: GitHub
    user: jonathanfilip
    repo: vim-lucius

  - provider: GitHub
    user: junegunn
    repo: fzf.vim

  - provider: GitHub
    user: junegunn
    repo: vim-easy-align

  - provider: GitHub
    user: justinmk
    repo: vim-dirvish

  - provider: GitHub
    user: kana
    repo: vim-textobj-user

  - provider: GitHub
    user: kh3phr3n
    repo: python-syntax

  - provider: GitHub
    user: lifepillar
    repo: vim-colortemplate

  - provider: GitHub
    user: lifepillar
    repo: vim-mucomplete

  - provider: GitHub
    user: othree
    repo: html5.vim

  - provider: GitHub
    user: pangloss
    repo: vim-javascript

  - provider: GitHub
    user: reedes
    repo: vim-textobj-quote

  - provider: GitHub
    user: romainl
    repo: vim-cool

  - provider: GitHub
    user: romainl
    repo: vim-qf

  - provider: GitHub
    user: rust-lang
    repo: rust.vim

  - provider: GitHub
    user: sgur
    repo: vim-editorconfig

  - provider: GitHub
    user: tmsvg
    repo: pear-tree

  - provider: GitHub
    user: tmux-plugins
    repo: vim-tmux

  - provider: GitHub
    user: tpope
    repo: vim-commentary

  - provider: GitHub
    user: tpope
    repo: vim-endwise

  - provider: GitHub
    user: tpope
    repo: vim-fugitive

  - provider: GitHub
    user: tpope
    repo: vim-git

  - provider: GitHub
    user: tpope
    repo: vim-markdown

  - provider: GitHub
    user: tpope
    repo: vim-repeat

  - provider: GitHub
    user: tpope
    repo: vim-surround

  - provider: GitHub
    user: tpope
    repo: vim-unimpaired

  - provider: GitHub
    user: wellle
    repo: targets.vim
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
