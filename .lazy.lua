vim.api.nvim_create_autocmd("FileType", {
	pattern = "rust",
	callback = function()
		vim.opt.colorcolumn = "100"
	end,
})

---@type LazySpec
return {}
