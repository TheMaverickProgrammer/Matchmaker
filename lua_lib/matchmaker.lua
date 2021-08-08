-- matchmaker lua module
local socket = require("socket")

local lib = {
    ip = "",          -- matchmaker server ip
    port = 0,         -- matchmaker server port
    timeout = 0,      -- connection timeout
    socket = nil,     -- udp socket
    session_key = "", -- active session key
    client_hash = ""  -- crypto hash of client to verify authenticity
}

local PacketId {
    PingPong = 0,
    Ack = 1,
    Create = 2,
    Join = 3 ,
    Close = 4,
    Error = 5    
}

function send_packet(ctx, packetId, data)
    local bytestream = nil

    -- {}
    if packetId == PacketId.PingPong then 
    
    end

    -- { id: u64 }
    if packetId == PacketId.Ack then

    end

    -- { client_hash: str, password_procted: bool }
    if packetId == PacketId.Create then 

    end

    -- { client_hash: str, session_key: str }
    if packetId == PacketId.Join then 

    end

    -- {}
    if packetId == PacketId.Close then 
    
    end
end

function read_packet(Ctx, bytestream)
    local packetId = 0 

    -- {}
    if packetId == PacketId.PingPong then 

    end

    -- { id: u64 }
    if packetId == PacketId.Ack then

    end

    -- { id: u64, message: str }
    if packetId == PacketId.Error then 

    end
end

function lib:check_config() 
    return string.len(self.ip) > 0 and self.port >= 1025 and self.port <= 65535 and string.len(self.client_hash) > 0 
end

function lib:init(client_hash, ip, port, timeout) 
    self.ip = ip
    self.port = port
    self.client_hash = client_hash

    if timeout ~= nil then
        self.timeout = timeout
    end

    if self:check_config() == false then
        print("Bad config")
    else
        if self.socket then 
            self.socket:close()
        end 

        self.socket = socket.udp()
        self.socket:setpeername(self.ip, self.port)
    end
end

function lib:create_session(password_protected)
    if self:check_config() then
        if string.len(self.session_key) == 0 then
            local data = {
                client_hash = self.client_hash,
                password_protected = password_protected
            }

            send_packet(self, PacketId.Create, data)
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
            
            send_packet(self, PacketId.Join, data)
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

        send_packet(self, PacketId.Close, {})
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

end

function lib:get_session() 
    return self.session_key
end

return lib
