-- Client
-- #!/usr/bin/env lua5.1

local wait_count = 10 -- in seconds

local dummy_hash = "YZ0123"
local mm = require("matchmaker")

mm:init(dummy_hash, '127.0.0.1', 3000, 1, true)

if mm:check_config() == false then return end

mm:create_session(true)

while(mm:get_session():len() == 0) do
    mm:poll()
end

print("Server returned session code: "..mm:get_session())

while(wait_count > 0) do
    wait_count = wait_count - 1
    print("wait_count: "..wait_count)
    mm:poll()
    mm:sleep(1.0)
end

-- mm.socket -- use this when connection is available
-- should also close the session on the server

mm:close() -- works!

print('Done')