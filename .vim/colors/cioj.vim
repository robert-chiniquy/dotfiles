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

hi TabLine	gui=bold guifg=#7b91e0 guibg=#030c33
hi TabLineFill guifg=#030c33 
hi TabLineSel gui=bold guibg=#7b91e0 guifg=#030c33
hi StatusLine gui=bold guibg=#7b91e0 guifg=#030c33
hi WildMenu guifg=#030c63

hi Normal		  ctermfg=111  ctermbg=0 guifg=#ffffff guibg=#000000 
hi Visual     guifg=#030c33 guibg=#fff9d9
hi Comment	  ctermfg=104 guifg=#7e73d1 guibg=#030c33
hi Constant	  ctermfg=15 guifg=#adbaf0 gui=italic term=bold
hi Special	  guifg=#ffe9e9 guibg=#030c33
hi Identifier gui=bold term=bold  ctermfg=147  guifg=#fefeef 
hi Statement  gui=bold term=bold  ctermfg=111  guifg=#7b91e0
hi PreProc	  term=underline  ctermfg=159  guifg=#bafffd
hi Type	      term=italic gui=italic ctermfg=159	 guifg=#fff9d9
hi Function	  gui=bold term=bold  ctermfg=189	guifg=#004bfa
hi Repeat	    term=bold	 ctermfg=189  guifg=#7e73e1 guibg=#030c33
hi Operator	  gui=bold term=bold guifg=#fff9d9
hi Ignore		  ctermfg=104  guifg=#7e73d1
hi Error      term=bold  ctermfg=230  guifg=#fff9d9
hi Todo	      term=bold gui=bold ctermfg=104 guifg=#000000 guibg=#fff9d9
hi javaScriptOperator guifg=#bafffd guibg=#030c33

" Common groups that link to default highlighting.
" You can specify other highlighting easily.
hi link String	Constant
hi link Character	Constant
hi link Number	Constant
hi link Boolean	Constant
hi link Float		Number
hi link Conditional	Repeat
hi link Label		Identifier
hi link Keyword	Repeat
hi link Exception	Special
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
hi link javaScriptIdentifier Repeat
hi link javaScriptFuncDef Identifier
hi link javaScriptOpSymbols Function
hi link javaScriptFuncKeyword Keyword
hi link javaScriptBraces Keyword
hi link javaScriptParens Function
hi link javaScriptConditional Function
hi link javaScriptLogicSymbols javaScriptOpSymbols
hi link javaScriptGlobalObjects Identifier
hi link javaScriptHtmlElemProperties Identifier
hi link javaScriptFuncArg Identifier
hi link javaScriptExceptions Keyword
hi link javaScriptOperator javaScriptOpSymbols
hi link javaScriptEndColons Function
hi link javaScriptLineComment Constant

