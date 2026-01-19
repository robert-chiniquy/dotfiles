-- bat-preview: Preview files with bat using vaporwave theme
local M = {}

function M:peek(job)
	local child = Command("bat")
		:args({
			"--color=always",
			"--style=plain",
			"--theme=vaporwave-custom",
			"--line-range",
			string.format("%d:%d", job.skip + 1, job.skip + job.area.h),
			tostring(job.file.url),
		})
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()

	if not child then
		return
	end

	local limit = job.area.h
	local lines, i = "", 0
	repeat
		local next, event = child:read_line()
		if event == 1 then
			break
		elseif event ~= 0 then
			break
		end

		i = i + 1
		if i > job.skip then
			lines = lines .. next
		end
	until i >= job.skip + limit

	child:start_kill()
	if job.skip > 0 and i < job.skip + limit then
		ya.manager_emit("peek", { math.max(0, i - limit), only_if = job.file.url, upper_bound = true })
	else
		ya.preview_widgets(job, { ui.Text.parse(lines):area(job.area) })
	end
end

function M:seek(job)
	local h = cx.active.current.hovered
	if h and h.url == job.file.url then
		local step = math.floor(job.units * job.area.h / 10)
		ya.manager_emit("peek", {
			math.max(0, cx.active.preview.skip + step),
			only_if = job.file.url,
		})
	end
end

return M
