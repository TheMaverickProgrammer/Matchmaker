-- matchmaker lua module
local socket = require("socket")
local serializer = require("serializer")

local lib = {
    ip = "",          -- matchmaker server ip
    port = 0,         -- matchmaker server port
    timeout = 0,      -- connection timeout
    socket = nil,     -- udp socket
    session_key = "", -- active session key (host only)
    remote_addr = "", -- remote connection
    client_hash = "", -- crypto hash of client to verify authenticity
    sent_packets = {},-- list of unack'd packaget that had been sent
    errors = {},      -- list of errors
    nextPacketId = 0, -- next packet ID
    debug = false     -- Prints debug information to console
}

--[[
Packet headers are u16 size
Packet IDs are u64 size
--]]
local PacketHeader = {
    PingPong = 0,
    Ack = 1,
    Create = 2,
    Join = 3 ,
    Close = 4,
    Error = 5    
}

function send_packet(ctx, packetId, header, data)
    serializer:clear()

    local littleEndian = serializer:endian() == "Little Endian"

    print("packetId: "..packetId)
    serializer:write_u64(packetId, false, littleEndian)

    -- print("header: "..header)

    serializer:write_u16(header, false, littleEndian)

    -- { id: u64 }
    if header == PacketHeader.Ack then
       serializer:write_u64(data.id, false, littleEndian)
    end

    -- { client_hash: str, password_protected: bool }
    if header == PacketHeader.Create then 
        -- print("Client hash: "..data.client_hash)
        serializer:write_string(data.client_hash, littleEndian)


        local value = 0

        if data.password_protected then
            value = 1
        end

        -- print("Password protected: "..value)
        serializer:write_u8(data.password_protected)
    end

    -- { client_hash: str, session_key: str }
    if header == PacketHeader.Join then 
        serializer:write_string(data.client_hash, littleEndian)
        serializer:write_string(data.session_key, littleEndian)
    end

    -- {}
    --[[
    if header == PacketHeader.PingPong then 
    end
    --]]

    -- {}
    --[[
    if header == PacketHeader.Close then
    end
    --]]

    ctx.nextPacketId = packetId + 1
    ctx.socket:send(serializer.Buffer)
    ctx.sent_packets[packetId] = serializer.Buffer
end

function read_packet(ctx, bytestream)
    serializer:set_buffer(bytestream)

    print("bystream is "..bytestream)

    local packetId = serializer:read_u64()

    local header = serializer:read_u16()

    print("header read was "..header)

    -- {}
    if header == PacketHeader.PingPong then 
        print("PingPong packet recieved")
        send_packet(ctx, ctx.nextPacketId, PacketHeader.PingPong, {})
    end

    -- { id: u64 }
    if header == PacketHeader.Ack then
        local id = serializer:read_u64()
        ctx.sent_packets[id] = nil
    end

    -- { id: u64, message: str }
    if header == PacketHeader.Error then 
        local id = serializer:read_u64()
        local message = serializer:read_string()
        ctx.sent_packets[id] = nil
        ctx.errors.push(message)
    end

    -- { session_key: str }
    if header == PacketHeader.Create then 
        local session_key = serializer:read_string()
        ctx.session_key = session_key
    end

    -- { socket_address: str }
    if header == PacketHeader.Join then 
        local socket_address = serializer:read_string()
        ctx.remote_addr = socket_address
    end
end

function lib:check_config() 
    return string.len(self.ip) > 0 
    and self.port >= 1025 
    and self.port <= 65535 
    and string.len(self.client_hash) > 0 
    and serializer:lua_byte_size() == 64
end

function lib:init(client_hash, ip, port, timeout, debug) 
    self.ip = ip
    self.port = port
    self.client_hash = client_hash

    if timeout ~= nil then
        self.timeout = timeout
    end

    if self:check_config() == false then
        if serializer:lua_byte_size() == 32 then 
            print("Matchmaker lib does not support 32bit architecture!")
        end
        
        print("Bad config")
    else
        if self.socket then 
            self.socket:close()
        end 

        self.socket = socket.udp()
        self.socket:setoption('reuseaddr',true)
        self.socket:setsockname('*', 3001)
        self.socket:setpeername(self.ip, self.port)

        self.nextPacketId = 0

        if debug == true then
            self.debug = true
            print("Host machine Endianess is "..serializer:endian())
        else 
            self.debug = false
        end
    end
end

function lib:create_session(password_protected)
    if self:check_config() then
        if string.len(self.session_key) == 0 then
            local data = {
                client_hash = self.client_hash,
                password_protected = password_protected
            }

            send_packet(self, self.nextPacketId, PacketHeader.Create, data)
        else 
            print("You have a session already @ "..self.session_key)
        end
    end
end

function lib:join_session(password)
    if self:check_config() then
        if string.len(self.session_key) == 0 then
            local data = {
                client_hash = self.client_hash,
                session_key = password
            }
            
            send_packet(self, self.nextPacketId, PacketHeader.Join, data)
        else 
            print("You are hosting a session, could not join a session!")
        end
    end
end

function lib:close_session() 
    if self:check_config() then
        if string.len(self.session_key) == 0 then 
            print("No session to close")
            return
        end

        send_packet(self, self.nextPacketId, PacketHeader.Close, {})
    end
end

function lib:close()
    if string.len(self.session_key) > 0 then 
        self:close_session()
    end

    if self.socket then 
        self.socket:close()
    end
end

-- Processes and acks incoming packets 
-- as well as resends drop packets
function lib:poll()
    local bytestream = self.socket:receive()

    if bytestream then 
        read_packet(self, bytestream)
    end
end

function lib:get_session() 
    return self.session_key
end

function lib:get_remote_addr()
    return self.remote_addr
end

return lib
