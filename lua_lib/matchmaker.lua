-- matchmaker lua module
local socket = require("socket")

local lib = {
    ip = "",          -- matchmaker server ip
    port = 0,         -- matchmaker server port
    timeout = 0,      -- udp socket timeout (TODO: remove)
    socket = nil,     -- udp socket
    session_key = "", -- active session key
    client_hash = ""  -- crypto hash of client to verify authenticity
}

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
        self.socket:settimeout(self.timeout)
    end
end

function lib:create_session(password_protected)
    if self:check_config() then
        if string.len(self.session_key) == 0 then
            command = self.client_hash .. " CREATE"

            if password_protected then 
                command = command .. " PASSWORD-ONLY"
            end

            self.socket:send(command)

            data = self.socket:receive()
            if data then
                print("Received: ", data)
                self.session_key = data
            end
        else 
            print("You have a session already @ "..self.session_key)
        end
    end
end

function lib:join_session(password)
    if self:check_config() then
        if string.len(self.session_key) == 0 then
            command = self.client_hash .. " JOIN "

            if password then 
                command = command .. " " .. password
            end

            self.socket:send(command)

            data = self.socket:receive()
            if data then
                print("Received: ", data)
            end
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

        command = "CLOSE"

        self.socket:send(command)

        data = self.socket:receive()
        if data then
            print("Received: ", data)

            -- clear session key
            self.session_key = ""
        end
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

return lib
