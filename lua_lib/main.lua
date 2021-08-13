-- Client
-- #!/usr/bin/env lua5.1

local dummy_hash = "YZ0123"
local mm = require("matchmaker")

mm:init(dummy_hash, 'localhost', 3000, 0, true)
mm:create_session(true)

while(mm:get_session():len() == 0) do
    mm:poll()
end

print("Server returned session code: "..mm:get_session())

--mm:join_session("8G9RnIe")
--mm:join_session()


-- should also close the session on the server
mm:close()

print('Done')