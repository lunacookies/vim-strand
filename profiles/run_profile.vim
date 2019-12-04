let s:vim_plug_dir = '~/.vim/plugged'

call plug#begin(s:vim_plug_dir)
    Plug 'tpope/vim-surround'
call plug#end()

let s:iters = 10

function! TestStrand() abort
    echo system('strand &> /dev/null')
endfunction

function! ClearVimPlugDir() abort
    echo system('rm -r ' . s:vim_plug_dir)
endfunction

function! TestVimPlug() abort
    PlugInstall --sync
endfunction

for i in range(s:iters)
    call TestStrand()
    call TestVimPlug()
    call ClearVimPlugDir()
endfor
