class First
end

class Second
end

local function total_values(values, enabled)
	local total = 0

	if enabled then
		total += 1
	else
		total -= 1
	end

	local step = 2

	for index = 1, #values do
		total += index
	end

	total += step

	for _, value in values do
		total += value
	end

	total += step

	while total < 10 do
		total += step
	end

	local finished = false

	repeat
		finished = true
	until finished

	total += step

	do
		total += 1
	end

	local helper = {}

	local function increase(value)
		return value + 1
	end

	function helper.decrease(value)
		return value - 1
	end

	total = helper.decrease(increase(total))

	return total
end

return total_values
