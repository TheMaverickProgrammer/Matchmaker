-- Client
-- #!/usr/bin/env lua5.1

local wait_count = 10 -- in seconds

local dummy_hash = "YZ0123"
local mm = require("matchmaker")

mm:init(dummy_hash, '127.0.0.1', 3000, 1, true)

if mm:check_config() == false then return end

--mm:join_session("nssiaA1")
mm:join_session()

while(wait_count > 0) do
    mm:poll()
    mm:sleep(1.0)
    wait_count = wait_count - 1
end

mm:close()

print('Done')