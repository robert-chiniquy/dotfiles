" Vim color file
" Name:       vaporwave.vim
" Based on:   cioj.vim color scheme
" Last Change: 2025 Jan 15

set background=dark
hi clear
if exists("syntax_on")
  syntax reset
endif
let g:colors_name = "vaporwave"

" === UI Elements ===
hi Normal         ctermfg=255  ctermbg=0   guifg=#ffffff guibg=#000000
hi Visual         guifg=#000000 guibg=#ff00f8
hi Cursor         gui=NONE   guifg=#000000   guibg=#ff00f8
hi CursorLine     guibg=#1a0030
hi CursorColumn   guibg=#1a0030
hi LineNr         ctermfg=141  guifg=#aa00e8 guibg=#0a0010
hi CursorLineNr   ctermfg=201 gui=bold guifg=#ff00f8
hi NonText        ctermfg=141  guifg=#aa00e8
hi SpecialKey     ctermfg=201  guifg=#ff00f8

" === Status/Tab Lines ===
hi StatusLine     gui=bold guibg=#ff00f8 guifg=#000000
hi StatusLineNC   gui=NONE guibg=#1a0030 guifg=#aa00e8
hi TabLine        gui=NONE guifg=#aa00e8 guibg=#030c33
hi TabLineFill    guifg=#030c33
hi TabLineSel     gui=bold guibg=#ff00f8 guifg=#000000
hi VertSplit      guifg=#ff00f8 guibg=#000000
hi WildMenu       gui=bold guifg=#000000 guibg=#5cecff

" === Popup Menu ===
hi Pmenu          guifg=#ffffff guibg=#1a0030
hi PmenuSel       gui=bold guifg=#000000 guibg=#ff00f8
hi PmenuSbar      guibg=#aa00e8
hi PmenuThumb     guibg=#ff00f8

" === Search/Matching ===
hi Search         gui=NONE guifg=#000000 guibg=#fbb725
hi IncSearch      gui=bold guifg=#000000 guibg=#5cecff
hi MatchParen     gui=bold guifg=#ffffff guibg=#aa00e8

" === Diff ===
hi DiffAdd        ctermfg=255 ctermbg=22  guifg=#ffffff guibg=#003322
hi DiffChange     ctermfg=255 ctermbg=58  guifg=#ffffff guibg=#332200
hi DiffDelete     ctermfg=255 ctermbg=52  guifg=#ffffff guibg=#330022
hi DiffText       ctermfg=255 ctermbg=94  cterm=bold gui=bold guifg=#ffffff guibg=#553300

" === Folding ===
hi Folded         guifg=#ffb1fe guibg=#1a0030
hi FoldColumn     guifg=#aa00e8 guibg=#0a0010

" === Messages ===
hi ErrorMsg       gui=bold guifg=#ffffff guibg=#ff0066
hi WarningMsg     gui=bold guifg=#000000 guibg=#fbb725
hi ModeMsg        gui=bold guifg=#5cecff
hi MoreMsg        gui=bold guifg=#5cecff
hi Question       gui=bold guifg=#fbb725

" === Syntax Groups ===
" Comments - muted purple
hi Comment        ctermfg=141 guifg=#aa00e8

" Constants/Strings - gold
hi Constant       ctermfg=221 guifg=#fbb725
hi String         ctermfg=221 guifg=#fbb725 guibg=#1a0015
hi Character      ctermfg=221 guifg=#fbb725
hi Number         ctermfg=221 guifg=#fbb725
hi Boolean        ctermfg=221 guifg=#fbb725
hi Float          ctermfg=221 guifg=#fbb725

" Identifiers - light pink
hi Identifier     ctermfg=219 gui=NONE guifg=#ffb1fe
hi Function       ctermfg=51  gui=bold guifg=#5cecff

" Statements/Keywords - hot pink bold
hi Statement      ctermfg=201 gui=bold guifg=#ff00f8
hi Conditional    ctermfg=201 gui=bold guifg=#ff00f8
hi Repeat         ctermfg=201 gui=bold guifg=#ff00f8
hi Label          ctermfg=201 guifg=#ff00f8
hi Operator       ctermfg=51  gui=bold guifg=#5cecff
hi Keyword        ctermfg=201 gui=bold guifg=#ff00f8
hi Exception      ctermfg=201 guifg=#ff00f8

" Preprocessor - cyan
hi PreProc        ctermfg=51  guifg=#5cecff
hi Include        ctermfg=51  guifg=#5cecff
hi Define         ctermfg=51  guifg=#5cecff
hi Macro          ctermfg=51  guifg=#5cecff
hi PreCondit      ctermfg=51  guifg=#5cecff

" Types - light pink italic
hi Type           ctermfg=219 gui=italic guifg=#ffb1fe
hi StorageClass   ctermfg=201 guifg=#ff00f8
hi Structure      ctermfg=219 guifg=#ffb1fe
hi Typedef        ctermfg=219 guifg=#ffb1fe

" Special - purple
hi Special        ctermfg=141 guifg=#c080d0
hi SpecialChar    ctermfg=141 guifg=#c080d0 guibg=#1a0015
hi Tag            ctermfg=201 guifg=#ff00f8
hi Delimiter      ctermfg=255 guifg=#ffffff
hi SpecialComment ctermfg=141 gui=italic guifg=#c080d0
hi Debug          ctermfg=201 guifg=#ff00f8

" Underlined - gold underline
hi Underlined     ctermfg=221 gui=underline guifg=#fbb725

" Ignore
hi Ignore         ctermfg=0   guifg=#1a0030

" Error - white on red
hi Error          gui=bold guifg=#ffffff guibg=#ff0066

" Todo - black on gold bold
hi Todo           ctermfg=0 ctermbg=221 gui=bold guifg=#000000 guibg=#fbb725

" === Language-specific ===
" JavaScript
hi link javaScriptIdentifier Identifier
hi link javaScriptFuncDef    Function
hi link javaScriptOpSymbols  Operator
hi link javaScriptFuncKeyword Keyword
hi link javaScriptBraces     Delimiter
hi link javaScriptParens     Delimiter
hi link javaScriptConditional Conditional
hi link javaScriptLogicSymbols Operator
hi link javaScriptGlobalObjects Type
hi link javaScriptOperator   Operator

" TypeScript
hi link typescriptBraces     Delimiter
hi link typescriptParens     Delimiter
hi link typescriptEndColons  Delimiter

" Go
hi link goDeclaration        Keyword
hi link goBuiltins           Function
hi link goConstants          Constant

" Python
hi link pythonBuiltin        Function
hi link pythonDecorator      PreProc
hi link pythonDecoratorName  Function

" Markdown
hi markdownH1         gui=bold guifg=#ff00f8
hi markdownH2         gui=bold guifg=#5cecff
hi markdownH3         gui=bold guifg=#ffb1fe
hi markdownH4         guifg=#fbb725
hi markdownCode       guifg=#fbb725 guibg=#1a0015
hi markdownCodeBlock  guifg=#fbb725 guibg=#1a0015
hi markdownUrl        guifg=#5cecff gui=underline
hi markdownLink       guifg=#ff00f8

" Spell checking
hi SpellBad       gui=undercurl guisp=#ff0066
hi SpellCap       gui=undercurl guisp=#5cecff
hi SpellRare      gui=undercurl guisp=#aa00e8
hi SpellLocal     gui=undercurl guisp=#fbb725

" vim: set et ts=2 sw=2 :
