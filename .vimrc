set background=dark
highlight NonText guifg=black

set runtimepath^=~/.vim/bundle/ctrlp.vim
set runtimepath^=~/.vim/bundle/taglist.vim
set runtimepath^=~/.vim/bundle/vim-fugitive
set runtimepath^=~/.vim/bundle/ag.vim

syntax on
syntax enable
filetype plugin indent on

set statusline+=%{fugitive#statusline()}

set history=1000

set ts=2
set whichwrap+=<,>,h,l,[,]
set modeline
set laststatus=2
set showmode
set title
set showcmd

set tags=./tags;/

let Tlist_Use_Right_Window=1
let Tlist_Auto_Open=0
let Tlist_Enable_Fold_Column=0
let Tlist_Compact_Format=0
let Tlist_WinWidth=28
let Tlist_Exit_OnlyWindow=1
let Tlist_File_Fold_Auto_Close = 1
nmap <LocalLeader>tt :Tlist<cr>

autocmd BufReadPre SConstruct set filetype=python

if has("gui_macvim")

  color cioj
  map <D-f> :set invfu<CR>   
  let g:nerdtree_tabs_open_on_gui_startup=0
  let NERDTreeShowHidden=1
  set guifont="Anonymous Pro":h14
  set ai
  set vb
  highlight WhitespaceEOL ctermbg=lightgray guibg=lightgray
  match WhitespaceEOL /s+$/

  set selection=exclusive
  set showmatch
  set backspace=indent,eol,start
  set laststatus=2
  set expandtab
  set shiftwidth=2
  set showtabline=1

  set guioptions=ce
  set guioptions=-r
  set mousef

  let macvim_hig_shift_movement = 1
  let g:ctrlp_working_path_mode = 'rc'
  set hlsearch
  set lcs=tab:▸\ ,trail:·,nbsp:_
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

