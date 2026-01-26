-- Neovim config
-- Sources existing .vimrc for backwards compatibility, then adds modern features

-- === Load existing vim config ===
vim.cmd('source ~/.vimrc')

-- === Nvim-specific settings ===
vim.opt.termguicolors = true      -- True color support
vim.opt.updatetime = 250          -- Faster completion
vim.opt.signcolumn = 'yes'        -- Always show signcolumn
vim.opt.undofile = true           -- Persistent undo
vim.opt.mouse = 'a'               -- Mouse support (optional, remove if you hate it)

-- === Bootstrap lazy.nvim (plugin manager) ===
local lazypath = vim.fn.stdpath('data') .. '/lazy/lazy.nvim'
if not vim.loop.fs_stat(lazypath) then
  vim.fn.system({
    'git', 'clone', '--filter=blob:none',
    'https://github.com/folke/lazy.nvim.git',
    '--branch=stable', lazypath,
  })
end
vim.opt.rtp:prepend(lazypath)

-- === Plugins ===
require('lazy').setup({
  -- Colorscheme (vaporwave-ish)
  {
    'folke/tokyonight.nvim',
    lazy = false,
    priority = 1000,
    config = function()
      require('tokyonight').setup({
        style = 'night',
        on_colors = function(colors)
          colors.hint = '#5cecff'
          colors.info = '#5cecff'
        end,
        on_highlights = function(hl, c)
          hl.Function = { fg = '#5cecff', bold = true }
          hl.Keyword = { fg = '#ff00f8', bold = true }
          hl.String = { fg = '#fbb725' }
          hl.Type = { fg = '#ffb1fe', italic = true }
          hl.Comment = { fg = '#aa00e8', italic = true }
        end,
      })
      vim.cmd('colorscheme tokyonight')
    end,
  },

  -- Treesitter (AST-based syntax highlighting)
  {
    'nvim-treesitter/nvim-treesitter',
    build = ':TSUpdate',
    config = function()
      -- New API: install parsers, highlighting is automatic in nvim 0.11+
      require('nvim-treesitter').setup({
        ensure_installed = { 'rust', 'go', 'lua', 'python', 'javascript', 'typescript', 'json', 'yaml', 'markdown', 'bash', 'toml' },
      })
    end,
  },

  -- Mason (LSP installer) + LSP config
  {
    'williamboman/mason.nvim',
    config = function()
      require('mason').setup()
    end,
  },
  {
    'neovim/nvim-lspconfig',
    dependencies = {
      'williamboman/mason.nvim',
      'williamboman/mason-lspconfig.nvim',
    },
    config = function()
      require('mason-lspconfig').setup({
        ensure_installed = { 'rust_analyzer', 'gopls', 'lua_ls', 'pyright', 'ts_ls' },
        automatic_installation = true,
        handlers = {
          function(server_name)
            require('lspconfig')[server_name].setup({})
          end,
        },
      })
    end,
  },

  -- Completion
  {
    'hrsh7th/nvim-cmp',
    dependencies = {
      'hrsh7th/cmp-nvim-lsp',
      'hrsh7th/cmp-buffer',
      'hrsh7th/cmp-path',
      'hrsh7th/cmp-nvim-lsp-signature-help',  -- Function signatures as you type
      'L3MON4D3/LuaSnip',  -- Required by nvim-cmp for LSP snippet expansion
    },
    config = function()
      local cmp = require('cmp')
      local luasnip = require('luasnip')

      cmp.setup({
        snippet = {
          expand = function(args)
            luasnip.lsp_expand(args.body)
          end,
        },
        mapping = cmp.mapping.preset.insert({
          ['<C-Space>'] = cmp.mapping.complete(),
          ['<CR>'] = cmp.mapping.confirm({ select = true }),
          ['<C-u>'] = cmp.mapping.scroll_docs(-4),
          ['<C-d>'] = cmp.mapping.scroll_docs(4),
          ['<Tab>'] = cmp.mapping(function(fallback)
            if cmp.visible() then
              cmp.select_next_item()
            else
              fallback()
            end
          end, { 'i', 's' }),
          ['<S-Tab>'] = cmp.mapping(function(fallback)
            if cmp.visible() then
              cmp.select_prev_item()
            else
              fallback()
            end
          end, { 'i', 's' }),
        }),
        sources = cmp.config.sources({
          { name = 'nvim_lsp', priority = 1000 },
          { name = 'nvim_lsp_signature_help' },
        }, {
          { name = 'buffer', keyword_length = 3 },
          { name = 'path' },
        }),
        formatting = {
          format = function(entry, vim_item)
            vim_item.menu = ({
              nvim_lsp = '[LSP]',
              buffer = '[Buf]',
              path = '[Path]',
            })[entry.source.name]
            return vim_item
          end,
        },
        window = {
          completion = cmp.config.window.bordered(),
          documentation = cmp.config.window.bordered(),
        },
      })
    end,
  },

  -- Telescope (fuzzy finder)
  {
    'nvim-telescope/telescope.nvim',
    branch = '0.1.x',
    dependencies = { 'nvim-lua/plenary.nvim' },
    config = function()
      local builtin = require('telescope.builtin')
      vim.keymap.set('n', '<leader>ff', builtin.find_files, {})
      vim.keymap.set('n', '<leader>fg', builtin.live_grep, {})
      vim.keymap.set('n', '<leader>fb', builtin.buffers, {})
      vim.keymap.set('n', '<C-p>', builtin.find_files, {})  -- Classic ctrl-p
    end,
  },

  -- Status line
  {
    'nvim-lualine/lualine.nvim',
    dependencies = { 'nvim-tree/nvim-web-devicons' },
    config = function()
      require('lualine').setup({
        options = { theme = 'tokyonight' },
      })
    end,
  },

  -- Git signs in gutter
  {
    'lewis6991/gitsigns.nvim',
    config = function()
      require('gitsigns').setup()
    end,
  },

  -- Better diagnostics display
  {
    'folke/trouble.nvim',
    dependencies = { 'nvim-tree/nvim-web-devicons' },
    config = function()
      vim.keymap.set('n', '<leader>xx', '<cmd>Trouble diagnostics toggle<cr>')
    end,
  },
}, {
  -- Don't pop up UI on startup after initial install
  change_detection = { enabled = false },
})

-- === Keybindings ===
vim.g.mapleader = '\\'  -- Match your existing vim leader

-- Quick save
vim.keymap.set('n', '<leader>w', ':w<CR>')

-- Clear search highlight
vim.keymap.set('n', '<Esc>', ':noh<CR>', { silent = true })

-- === LSP Keybindings ===
vim.api.nvim_create_autocmd('LspAttach', {
  callback = function(args)
    local opts = { buffer = args.buf }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
    vim.keymap.set('n', '<leader>f', function() vim.lsp.buf.format({ async = true }) end, opts)
  end,
})

-- === Highly Supported: Go & Rust ===
-- Format on save for Go and Rust
vim.api.nvim_create_autocmd('BufWritePre', {
  pattern = { '*.go', '*.rs' },
  callback = function()
    vim.lsp.buf.format({ async = false })
  end,
})

-- Go: organize imports on save
vim.api.nvim_create_autocmd('BufWritePre', {
  pattern = '*.go',
  callback = function()
    local params = vim.lsp.util.make_range_params()
    params.context = { only = { 'source.organizeImports' } }
    local result = vim.lsp.buf_request_sync(0, 'textDocument/codeAction', params, 1000)
    for _, res in pairs(result or {}) do
      for _, r in pairs(res.result or {}) do
        if r.edit then
          vim.lsp.util.apply_workspace_edit(r.edit, 'utf-8')
        elseif r.command then
          vim.lsp.buf.execute_command(r.command)
        end
      end
    end
  end,
})
