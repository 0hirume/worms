const FIRST = 1

const OPTIONS = configure({
	enabled = true,
})

const LAST = 2

local function build()
	const template = read_template()

	const primary = create()
	primary.name = "primary"
	primary["enabled"] = true

	const secondary = create()
	secondary.name = "secondary"
	primary.child = secondary
	secondary.parent = primary

	return secondary
end

return build
