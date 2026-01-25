# Nushell Configuration
# Ported from zsh config - vaporwave theme
# version = "0.110.0"

# === Vaporwave Color Theme ===
# Hot pink: #ff0099 / 201
# Cyan: #5cecff / 51
# Magenta: #ff00f8
# Gold: #fbb725 / 221
# Purple: #aa00e8 / 129

$env.config = {
    show_banner: false

    ls: {
        use_ls_colors: true
        clickable_links: true
    }

    rm: {
        always_trash: false
    }

    table: {
        mode: rounded
        index_mode: always
        show_empty: true
        padding: { left: 1 right: 1 }
        trim: {
            methodology: wrapping
            wrapping_try_keep_words: true
        }
        header_on_separator: false
    }

    error_style: "fancy"

    history: {
        max_size: 50000
        sync_on_enter: true
        file_format: "sqlite"
        isolation: false
    }

    completions: {
        case_sensitive: false
        quick: true
        partial: true
        algorithm: "fuzzy"
        use_ls_colors: true
    }

    filesize: {
        unit: "binary"
    }

    cursor_shape: {
        emacs: line
        vi_insert: line
        vi_normal: block
    }

    color_config: {
        separator: { fg: '#aa00e8' }
        leading_trailing_space_bg: { attr: n }
        header: { fg: '#5cecff' attr: b }
        empty: { fg: '#5cecff' }
        bool: { fg: '#ffb1fe' }
        int: { fg: '#fbb725' }
        filesize: { fg: '#5cecff' }
        duration: { fg: '#fbb725' }
        date: { fg: '#aa00e8' }
        range: { fg: '#fbb725' }
        float: { fg: '#fbb725' }
        string: { fg: '#fbb725' }
        nothing: { fg: '#666666' }
        binary: { fg: '#aa00e8' }
        cell_path: { fg: '#5cecff' }
        row_index: { fg: '#5cecff' attr: b }
        record: { fg: '#5cecff' }
        list: { fg: '#ffb1fe' }
        block: { fg: '#5cecff' }
        hints: dark_gray
        search_result: { bg: '#ff00f8' fg: white }

        # Syntax shapes (live typing colors)
        shape_binary: { fg: '#aa00e8' attr: b }
        shape_block: { fg: '#5cecff' attr: b }
        shape_bool: { fg: '#ffb1fe' }
        shape_closure: { fg: '#ff00f8' attr: b }
        shape_custom: { fg: '#5cecff' }
        shape_datetime: { fg: '#5cecff' attr: b }
        shape_directory: { fg: '#5cecff' }
        shape_external: { fg: '#5cecff' }
        shape_externalarg: { fg: '#fbb725' }
        shape_external_resolved: { fg: '#5cecff' attr: b }
        shape_filepath: { fg: '#fbb725' attr: u }
        shape_flag: { fg: '#5cecff' attr: b }
        shape_float: { fg: '#fbb725' attr: b }
        shape_glob_interpolation: { fg: '#5cecff' attr: b }
        shape_globpattern: { fg: '#5cecff' attr: b }
        shape_int: { fg: '#fbb725' attr: b }
        shape_internalcall: { fg: '#ff00f8' attr: b }
        shape_keyword: { fg: '#ff00f8' attr: b }
        shape_list: { fg: '#ffb1fe' attr: b }
        shape_literal: { fg: '#5cecff' }
        shape_match_pattern: { fg: '#5cecff' }
        shape_matching_brackets: { attr: u }
        shape_nothing: { fg: '#666666' }
        shape_operator: { fg: '#02c3fc' attr: b }
        shape_pipe: { fg: '#aa00e8' attr: b }
        shape_range: { fg: '#fbb725' attr: b }
        shape_record: { fg: '#5cecff' attr: b }
        shape_redirection: { fg: '#aa00e8' attr: b }
        shape_signature: { fg: '#5cecff' attr: b }
        shape_string: { fg: '#fbb725' }
        shape_string_interpolation: { fg: '#5cecff' attr: b }
        shape_table: { fg: '#5cecff' attr: b }
        shape_variable: { fg: '#aa00e8' }
        shape_vardecl: { fg: '#aa00e8' }
        shape_raw_string: { fg: '#ffb1fe' }
        shape_garbage: { fg: '#ff0000' bg: '#330000' attr: b }
    }

    keybindings: [
        # Ctrl+P: Fuzzy history search (like zsh)
        {
            name: fuzzy_history
            modifier: control
            keycode: char_p
            mode: [emacs vi_normal vi_insert]
            event: {
                send: executehostcommand
                cmd: "commandline edit (history | get command | reverse | uniq | str join (char nl) | fzf --height=40% --prompt='History: ' --no-sort)"
            }
        }
        # Ctrl+T: Fuzzy file search
        {
            name: fuzzy_file
            modifier: control
            keycode: char_t
            mode: [emacs vi_normal vi_insert]
            event: {
                send: executehostcommand
                cmd: "commandline edit --insert (fd --type f --hidden --follow --exclude .git | fzf --preview 'bat --color=always --style=numbers --line-range=:50 {}' --preview-window=right:60%:wrap)"
            }
        }
        # Alt+C: Fuzzy cd
        {
            name: fuzzy_cd
            modifier: alt
            keycode: char_c
            mode: [emacs vi_normal vi_insert]
            event: {
                send: executehostcommand
                cmd: "cd (fd --type d --hidden --follow --exclude .git | fzf --preview 'eza --tree --level=1 --color=always {}')"
            }
        }
    ]

    hooks: {
        pre_prompt: [
            {||
                # Direnv integration
                if (which direnv | is-not-empty) {
                    direnv export json | from json | default {} | load-env
                }
            }
        ]
        env_change: {
            PWD: [
                {|before, after|
                    # Auto-ls: show tree if small enough directory
                    let item_count = (ls -a | length)
                    if $item_count <= 30 {
                        eza --tree --level=1 --icons --git --color=always --group-directories-first
                    }
                }
            ]
        }
    }
}

# === Source Tool Integrations ===
# These files should be generated by running:
#   starship init nu | save -f ($nu.default-config-dir | path join 'starship.nu')
#   zoxide init nushell | save -f ($nu.default-config-dir | path join 'zoxide.nu')
#   atuin init nu --disable-up-arrow | save -f ($nu.default-config-dir | path join 'atuin.nu')

source starship.nu
source zoxide.nu

# Atuin (shell history) - comment out if not using atuin
source atuin.nu

# === Aliases ===

# eza (better ls) - vaporwave colors set in env.nu via EZA_COLORS
alias ls = eza --icons --color=always --group-directories-first
alias ll = eza --icons --color=always --group-directories-first -l
alias la = eza --icons --color=always --group-directories-first -la
alias lt = eza --icons --color=always --tree --level=2
alias lsg = eza --icons --color=always --group-directories-first --git -l

# bat (better cat)
alias cat = bat --style=plain --paging=never
alias bcat = bat --style=full

# neovim
alias vim = nvim
alias vi = nvim

# Directory stack
alias d = dirs

# gbanner on macOS
alias banner = gbanner

# === Custom Functions ===

# glow: Render markdown with vaporwave theme
def glow [file?: path] {
    if ($file == null) {
        ^glow -p -s ~/.config/glow/vaporwave.json
    } else {
        ^glow -p -s ~/.config/glow/vaporwave.json $file
    }
}

# md: Render one or all markdown files
def md [...files: path] {
    if ($files | is-empty) {
        let md_files = (glob "*.md")
        if ($md_files | is-empty) {
            print -e "No markdown files in current directory"
            return
        }
        for f in $md_files {
            glow $f
        }
    } else {
        for f in $files {
            glow $f
        }
    }
}

# mdtoc: Show table of contents for markdown file
def mdtoc [file: path] {
    let toc = (open $file | lines | where {|l| $l =~ '^#{1,6} '} | str replace -r '^#' ' ' | str replace -r '^#' '  ' | str replace -r '^#' '   ')
    print $"(ansi { fg: '#ff0099' attr: b })======== TABLE OF CONTENTS ========"
    print ($toc | first 30 | str join "\n")
    print $"(ansi { fg: '#ff0099' attr: b })===================================="
    print ""
    glow $file
}

# tree: Smart directory tree (uses erd if available, falls back to eza)
def tree [path?: path = ".", --level (-l): int = 2] {
    if (which erd | is-not-empty) {
        ^erd --icons --human --level $level --sort name --dir-order first --color force ($path)
    } else {
        eza --icons --color=always --tree --level=($level) ($path)
    }
}

# jqc: JSON viewer with colors and pager
def jqc [] { $in | jq -C | less -R }

# yqc: YAML viewer with colors and pager
def yqc [] { $in | yq -C | less -R }

# === Yazi Integration ===
# Start yazi and cd to the directory it exits in
def --env y [...args] {
    let tmp = (mktemp -t "yazi-cwd.XXXXXX")
    ^yazi --cwd-file $tmp ...$args
    let cwd = (open $tmp | str trim)
    if ($cwd | is-not-empty) and ($cwd != $env.PWD) {
        cd $cwd
    }
    rm -f $tmp
}

# === Markdown Browser ===
# Use: mdb to browse markdown files with fzf
def mdb [] {
    let md_files = (glob "**/*.md" | where {|f| not ($f | str contains "node_modules")} | where {|f| not ($f | str contains ".git")} | first 150)

    if ($md_files | is-empty) {
        print "No markdown files found"
        return
    }

    let selected = ($md_files | str join "\n" | fzf --preview 'bat --color=always --style=plain --language=md {}' --preview-window=right:75%:wrap --height=100% --border=rounded)

    if ($selected | is-not-empty) {
        glow $selected
    }
}
