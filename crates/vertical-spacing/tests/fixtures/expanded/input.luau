register(
"Alpha",
{
value = 1,
}
)
register(
"Beta",
{
value = 2,
}
)
local FIRST = {
enabled = true,
}
local SECOND = {
enabled = false,
}
state.options = {
mode = "fast",
}
state.result = calculate(
"the first value is intentionally long enough to keep this call expanded",
"the second value is intentionally long enough to keep this call expanded"
)
state.handler = function(value)
return value
end
state.ready = true
type Detailed = {
enabled: boolean,
name: string,
}
type Done = boolean
return state
