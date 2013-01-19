set nocompatible

set background=dark
highlight NonText guifg=black

set runtimepath^=~/.vim/bundle/ctrlp.vim
set runtimepath^=~/.vim/bundle/taglist.vim
set runtimepath^=~/.vim/bundle/vim-fugitive
set runtimepath^=~/.vim/bundle/ag.vim
set runtimepath^=/.vim/bundle/supertab

syntax on
syntax enable
filetype plugin indent on

set statusline=%#identifier#
set statusline+=%m
set statusline+=%*
set statusline+=%{fugitive#statusline()}
set statusline+=%=
set statusline+=%l/%L
set statusline+=%#TabLine#
set statusline+=[%f]
set statusline+=%*
set statusline+=%#TabLine#
set statusline+=%y
"set statusline+=%{Tlist_Get_Tagname_By_Line()}
"set statusline+=%{Tlist_Get_Tag_Prototype_By_Line()}
set statusline+=%*

set history=1000

set ts=2
set whichwrap+=<,>,h,l,[,]
set modeline
set laststatus=2
set showcmd
set showmode
set title
set autoread

set tags=./tags;/

let Tlist_Ctags_Cmd = '/usr/local/bin/ctags'
let Tlist_Use_Right_Window=1
let Tlist_Auto_Open=0
let Tlist_Enable_Fold_Column=0
let Tlist_Compact_Format=1
let Tlist_WinWidth=28
let Tlist_Exit_OnlyWindow=1
let Tlist_File_Fold_Auto_Close = 1
let Tlist_Auto_Highlight_Tag=1
let Tlist_File_Fold_Auto_Close=1
let Tlist_Inc_Winwidth=1
let Tlist_Use_SingleClick=1

nmap <LocalLeader>tt :Tlist<cr>

nmap <LocalLeader>gb :Gblame<cr>
nmap <LocalLeader>gd :Gdiff<cr>
nmap <LocalLeader>gs :Gstatus<cr><C-w>20+
nmap <LocalLeader>gc :Gcommit<cr>

autocmd BufReadPre SConstruct set filetype=python

if has("gui_macvim")

  color cioj
  set fullscreen
  set fuoptions=maxvert,maxhorz
  map <D-f> :set invfu<CR>

  set guifont=Anonymous\ Pro:h14
  set ai
  set vb
  set list
  set listchars=tab:▷⋅,trail:⋅,nbsp:⋅
  set lcs=tab:▸\ ,trail:·,nbsp:_
  highlight WhitespaceEOL ctermbg=lightgray guibg=lightgray
  match WhitespaceEOL /s\+$/

  set showmatch
  set switchbuf=useopen
  set backspace=indent,eol,start
  set expandtab
  set shiftwidth=2
  set showtabline=1
  set cmdheight=1
  set guioptions=c
  set guioptions-=r
  set mousef

  set selection=exclusive
  let macvim_hig_shift_movement = 1
  let g:ctrlp_working_path_mode = 'rc'
  set hlsearch

  set clipboard=unnamed
  set incsearch

  set magic

  map <leader>cc :botright cope<cr>
  map <leader>co ggVGy:tabnew<cr>:set syntax=qf<cr>pgg
  map <leader>n :cn<cr>
  map <leader>p :cp<cr>

  vnoremap <Tab> 1>
  vnoremap <S-Tab> 1<
  nnoremap <Tab> >>
  nnoremap <S-Tab> <<
  map <C-\> :tab split<CR>:exec("tag ".expand("<cword>"))<CR>
  
  map <silent><D-t> :tabnew<CR>

else

  color inkpot

endif

map <F10> :echo "hi<" . synIDattr(synID(line("."),col("."),1),"name") . '> trans<' . synIDattr(synID(line("."),col("."),0),"name") . "> lo<" . synIDattr(synIDtrans(synID(line("."),col("."),1)),"name") . ">" <CR>

autocmd FileType javascript set omnifunc=javascriptcomplete#CompleteJS
