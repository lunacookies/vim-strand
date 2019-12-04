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
  - GitHub:
      user: PeterRincker
      repo: vim-searchlight

  - GitHub:
      user: cakebaker
      repo: scss-syntax.vim

  - GitHub:
      user: cespare
      repo: vim-toml

  - GitHub:
      user: christoomey
      repo: vim-tmux-navigator

  - GitHub:
      user: cocopon
      repo: inspecthi.vim

  - GitHub:
      user: hail2u
      repo: vim-css3-syntax

  - GitHub:
      user: jonathanfilip
      repo: vim-lucius

  - GitHub:
      user: junegunn
      repo: fzf.vim

  - GitHub:
      user: junegunn
      repo: vim-easy-align

  - GitHub:
      user: justinmk
      repo: vim-dirvish

  - GitHub:
      user: kana
      repo: vim-textobj-user

  - GitHub:
      user: kh3phr3n
      repo: python-syntax

  - GitHub:
      user: lifepillar
      repo: vim-colortemplate

  - GitHub:
      user: lifepillar
      repo: vim-mucomplete

  - GitHub:
      user: othree
      repo: html5.vim

  - GitHub:
      user: pangloss
      repo: vim-javascript

  - GitHub:
      user: reedes
      repo: vim-textobj-quote

  - GitHub:
      user: romainl
      repo: vim-cool

  - GitHub:
      user: romainl
      repo: vim-qf

  - GitHub:
      user: rust-lang
      repo: rust.vim

  - GitHub:
      user: sgur
      repo: vim-editorconfig

  - GitHub:
      user: tmsvg
      repo: pear-tree

  - GitHub:
      user: tmux-plugins
      repo: vim-tmux

  - GitHub:
      user: tpope
      repo: vim-commentary

  - GitHub:
      user: tpope
      repo: vim-endwise

  - GitHub:
      user: tpope
      repo: vim-fugitive

  - GitHub:
      user: tpope
      repo: vim-git

  - GitHub:
      user: tpope
      repo: vim-markdown

  - GitHub:
      user: tpope
      repo: vim-repeat

  - GitHub:
      user: tpope
      repo: vim-surround

  - GitHub:
      user: tpope
      repo: vim-unimpaired

  - GitHub:
      user: wellle
      repo: targets.vim
" > "$strand_config"

# Install vim-plug before running the profile
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
