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
      require('nvim-treesitter.configs').setup({
        ensure_installed = { 'rust', 'go', 'lua', 'python', 'javascript', 'typescript', 'json', 'yaml', 'markdown', 'bash', 'toml' },
        auto_install = true,
        highlight = { enable = true },
        indent = { enable = true },
      })
    end,
  },

  -- LSP (Language Server Protocol)
  {
    'neovim/nvim-lspconfig',
    dependencies = {
      'williamboman/mason.nvim',
      'williamboman/mason-lspconfig.nvim',
    },
    config = function()
      require('mason').setup()
      require('mason-lspconfig').setup({
        ensure_installed = { 'rust_analyzer', 'gopls', 'lua_ls', 'pyright', 'ts_ls' },
        automatic_installation = true,
      })

      local lspconfig = require('lspconfig')
      local on_attach = function(_, bufnr)
        local opts = { buffer = bufnr }
        vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
        vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
        vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
        vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
        vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
      end

      require('mason-lspconfig').setup_handlers({
        function(server_name)
          lspconfig[server_name].setup({ on_attach = on_attach })
        end,
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
      'L3MON4D3/LuaSnip',
      'saadparwaiz1/cmp_luasnip',
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
          ['<Tab>'] = cmp.mapping(function(fallback)
            if cmp.visible() then
              cmp.select_next_item()
            elseif luasnip.expand_or_jumpable() then
              luasnip.expand_or_jump()
            else
              fallback()
            end
          end, { 'i', 's' }),
        }),
        sources = cmp.config.sources({
          { name = 'nvim_lsp' },
          { name = 'luasnip' },
        }, {
          { name = 'buffer' },
          { name = 'path' },
        }),
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
})

-- === Keybindings ===
vim.g.mapleader = '\\'  -- Match your existing vim leader

-- Quick save
vim.keymap.set('n', '<leader>w', ':w<CR>')

-- Clear search highlight
vim.keymap.set('n', '<Esc>', ':noh<CR>', { silent = true })
