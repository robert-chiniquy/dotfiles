" local syntax file - set colors on a per-machine basis:
" vim: tw=0 ts=4 sw=4
" Vim color file
" Designed by:	Tony Quintero
" Last Change:	2013 Jan 07

set background=dark
hi clear
if exists("syntax_on")
  syntax reset
endif
let g:colors_name = "cioj"
hi Normal		  ctermfg=111  ctermbg=17  guifg=#7b91e0	 guibg=#030c33
hi Comment	  ctermfg=104 guifg=#7e73d1
hi Constant	  term=bold
hi Special	  ctermfg=230  guifg=#fff9d9
hi Identifier term=bold  ctermfg=147  guifg=#adbaf0
hi Statement  term=bold  ctermfg=111  guifg=#7b91e0
hi PreProc	  term=underline  ctermfg=159  guifg=#bafffd
hi Type	      term=underline	ctermfg=159	 guifg=#bafffd
hi Function	  term=bold  ctermfg=189	guifg=#eeeeff
hi Repeat	    term=bold	 ctermfg=189  guifg=#eeeeff
hi Operator	  term=bold
hi Ignore		  ctermfg=104  guifg=#7e73d1
hi Error      term=bold  ctermfg=230  guifg=#fff9d9
hi Todo	      ctermfg=230 guifg=#fff9d9

" Common groups that link to default highlighting.
" You can specify other highlighting easily.
hi link String	Constant
hi link Character	Constant
hi link Number	Constant
hi link Boolean	Constant
hi link Float		Number
hi link Conditional	Repeat
hi link Label		Statement
hi link Keyword	Statement
hi link Exception	Statement
hi link Include	PreProc
hi link Define	PreProc
hi link Macro		PreProc
hi link PreCondit	PreProc
hi link StorageClass	Type
hi link Structure	Type
hi link Typedef	Type
hi link Tag		Special
hi link SpecialChar	Special
hi link Delimiter	Special
hi link SpecialComment Special
hi link Debug		Special
