# Nushell Environment Configuration
# Ported from zsh config - vaporwave theme

# === PATH Setup ===
$env.PATH = ($env.PATH | split row (char esep)
    | prepend '/opt/homebrew/bin'
    | prepend '/opt/homebrew/sbin'
    | prepend $'($nu.home-dir)/go/bin'
    | prepend $'($nu.home-dir)/.opencode/bin'
    | uniq)

# === Environment Variables ===
$env.EDITOR = 'vim'
$env.VISUAL = 'vim'
$env.CLICOLOR = '1'
$env.HOMEBREW_NO_ENV_HINTS = 'true'

# Starship logging
$env.STARSHIP_LOG = 'error'

# Ripgrep config
$env.RIPGREP_CONFIG_PATH = $'($nu.home-dir)/.ripgreprc'

# Difftastic (structural diff)
$env.DFT_BACKGROUND = 'dark'
$env.DFT_COLOR = 'always'

# Vaporwave FZF Colors
$env.FZF_COLORS = 'fg:#ffffff,bg:#000000,hl:#ff00f8,fg+:#000000,bg+:#ff00f8,hl+:#5cecff,info:#5cecff,border:#ff00f8,prompt:#ffb1fe,pointer:#5cecff,marker:#ff00f8,spinner:#5cecff,header:#aa00e8'

# FZF defaults
$env.FZF_CTRL_T_COMMAND = 'fd --type f --hidden --follow --exclude .git --exclude cache --exclude plugin --exclude plugins'
$env.FZF_CTRL_T_OPTS = $"--preview 'if file -b --mime {} | grep -q text; then bat --color=always --style=numbers --line-range=:500 {}; else xxd -l 512 {}; fi' --preview-window=right:60%:wrap --color=($env.FZF_COLORS)"
$env.FZF_ALT_C_COMMAND = 'fd --type d --hidden --follow --exclude .git --exclude cache --exclude plugin --exclude plugins'
$env.FZF_ALT_C_OPTS = $"--preview 'eza --tree --level=1 --color=always {}' --color=($env.FZF_COLORS)"

# Colorized grep output
$env.GREP_COLORS = 'ms=01;38;5;201:mc=01;38;5;51:sl=:cx=:fn=38;5;221:ln=38;5;51:bn=38;5;51:se=38;5;201'

# Vaporwave LS_COLORS (used by various tools)
$env.LS_COLORS = 'di=1;38;5;51:ln=38;5;129:so=38;5;129:pi=38;5;221:ex=1;38;5;129:bd=38;5;175:cd=38;5;175:su=38;5;129;48;5;51:sg=38;5;51;48;5;175:tw=38;5;51;48;5;175:ow=38;5;51:*.rs=1;38;5;129:*.go=38;5;51:*.py=38;5;221:*.js=38;5;221:*.ts=38;5;51:*.tsx=38;5;51:*.md=38;5;129:*.json=38;5;221:*.yaml=38;5;221:*.yml=38;5;221:*.toml=38;5;175:*.sh=38;5;51:*.zsh=38;5;51:*.lua=38;5;51:*.sql=38;5;221:*.html=38;5;221:*.css=38;5;51:*.proto=38;5;51:*.txt=38;5;129:*.log=38;5;243:*.git=1;38;5;175:*.gitignore=38;5;175:Makefile=1;38;5;129:Dockerfile=1;38;5;129:Cargo.toml=1;38;5;129:*.lock=38;5;175:*.tar=38;5;221:*.gz=38;5;221:*.zip=38;5;221:*.png=38;5;51:*.jpg=38;5;51:*.svg=38;5;51:*.mp4=38;5;129:*.mp3=38;5;129'

# EZA colors (vaporwave - retina-searing electric colors)
$env.EZA_COLORS = 'da=38;5;51:di=38;5;51;1:ex=38;5;201;1:ln=38;5;171:*.md=38;5;171:*.txt=38;5;171:*.rst=38;5;171:*.org=38;5;171:*.json=38;5;221:*.yaml=38;5;221:*.yml=38;5;221:*.xml=38;5;221:*.csv=38;5;221:*.go=38;5;51:*.rs=38;5;201:*.py=38;5;221:*.lua=38;5;51:*.ts=38;5;51:*.tsx=38;5;51:*.js=38;5;221:*.jsx=38;5;221:*.html=38;5;221:*.css=38;5;51:*.scss=38;5;51:*.sh=38;5;51:*.bash=38;5;51:*.zsh=38;5;51:*.fish=38;5;51:*.sql=38;5;221:*.graphql=38;5;171:*.toml=38;5;129:*.ini=38;5;129:*.conf=38;5;129:*.cfg=38;5;129:*.env=38;5;129:*.gitignore=38;5;129:*.dockerignore=38;5;129:*.editorconfig=38;5;129:Makefile=38;5;201:Dockerfile=38;5;201:Justfile=38;5;201:Cargo.toml=38;5;201:Cargo.lock=38;5;129:*.proto=38;5;51:*.pb.go=38;5;129:*.test.go=38;5;171:*_test.go=38;5;171:*.spec.ts=38;5;171:*.test.ts=38;5;171'

# Vaporwave man pages (less termcap)
$env.LESS_TERMCAP_mb = "\e[1;38;5;201m"      # begin bold (hot pink)
$env.LESS_TERMCAP_md = "\e[1;38;5;51m"       # begin blink (cyan)
$env.LESS_TERMCAP_me = "\e[0m"               # reset bold/blink
$env.LESS_TERMCAP_so = "\e[1;38;5;15;48;5;201m"  # reverse video (white on hot pink)
$env.LESS_TERMCAP_se = "\e[0m"               # reset reverse
$env.LESS_TERMCAP_us = "\e[1;38;5;221m"      # begin underline (gold)
$env.LESS_TERMCAP_ue = "\e[0m"               # reset underline

# === Prompt Configuration ===
# Using starship for prompt - disable built-in prompts
$env.PROMPT_COMMAND = {|| "" }
$env.PROMPT_COMMAND_RIGHT = {|| "" }
$env.PROMPT_INDICATOR = ""
$env.PROMPT_INDICATOR_VI_INSERT = ""
$env.PROMPT_INDICATOR_VI_NORMAL = ""
$env.PROMPT_MULTILINE_INDICATOR = "::: "

# === Carapace Bridges ===
$env.CARAPACE_BRIDGES = 'zsh,fish,bash,inshellisense'
