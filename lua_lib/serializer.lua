---Read Functions
local bit = require("bit")

local serializer = {
	Buffer = "",
	Position = 0
}

function serializer:endian()
	local function f() end
	
	if string.byte(string.dump(f),7) == 1 then 
		return "Little Endian"
	end

	return "Big Endian"
end

function serializer:clear() 
	self.Buffer = ""
	self.Position = 0
end

function serializer:set_buffer(buffer)
	self.Buffer = buffer
	self.Position = 0
end

function serializer:check_bytes(...)
	if type(arg) ~= "table" then
		print("arg type not a table in function `serializer:check_bytes(...)`")
	else
		local i = 0
		for _ in pairs(arg) do 
			i = i + 1
			if arg[i] == 0 then arg[i] = "\0" end
		end
	end

	return unpack(arg)
end

function serializer:read_u8() 
    self.Position = self.Position + 1
    return self.Buffer:byte(self.Position)
end

function serializer:read_u16(reversed)
 
	local l1,l2 = 0
	if not(reversed) then
     l1 = self:read_u8()
     l2 = bit.lshift(self:read_u8(), 8)

	else
     l1 = bit.lshift(self:read_u8(), 8)
     l2 = self:read_u8()	 
	end
	
    return bit.bor(l1, l2)
end
 
function serializer:read_u32(reversed)
	local l1,l2,l3,l4 = 0
	if not(reversed) then
     l1 = self:read_u8()
     l2 = bit.lshift(self:read_u8(), 8)
     l3 = bit.lshift(self:read_u8(), 16)
     l4 = bit.lshift(self:read_u8(), 24)
	else
     l1 = bit.lshift(self:read_u8(), 24)
     l2 = bit.lshift(self:read_u8(), 16)
     l3 = bit.lshift(self:read_u8(), 8)
     l4 = self:read_u8()	 
	end
	
    return bit.bor(l1, l2, l3, l4)
end

function serializer:read_u64(reversed)
	local l1,l2,l3,l4,l5,l6,l7,l8 = 0
	if not(reversed) then
     l1 = self:read_u8()
     l2 = bit.lshift(self:read_u8(), 8)
     l3 = bit.lshift(self:read_u8(), 16)
     l4 = bit.lshift(self:read_u8(), 24)
	 l5 = bit.lshift(self:read_u8(), 32)
	 l6 = bit.lshift(self:read_u8(), 40)
	 l7 = bit.lshift(self:read_u8(), 48)
	 l8 = bit.lshift(self:read_u8(), 56)
	else
     l1 = bit.lshift(self:read_u8(), 56)
     l2 = bit.lshift(self:read_u8(), 48)
     l3 = bit.lshift(self:read_u8(), 40)
     l4 = bit.lshift(self:read_u8(), 32)
	 l5 = bit.lshift(self:read_u8(), 24)
	 l6 = bit.lshift(self:read_u8(), 16)
	 l7 = bit.lshift(self:read_u8(), 8)
	 l8 = self:read_u8()
	end
	
    return bit.bor(l1, l2, l3, l4, l5, l6, l7, l8)
end

function serializer:read_string(reversed)
    local len = self:read_u16(reversed)
	local ret = ""
	if len == 0 then
	 return ret
	end
	for i = 0, len - 1 do
		local char = self:read_u8()
		if char == nil then break end
		ret = ret ..string.char(char)
	end
 
    return ret
end

--Write Functions
 
function serializer:write_u8(byte, insert)
    self.Position = self.Position + 1
	if type(byte) == "number" then
	   byte = string.char(byte)
	end
	if type(byte) == "boolean" then
		if byte == true then
			byte = 1
		else
			byte = 0
		end

		byte = string.char(byte)
	end

    if insert then
        self.Buffer = self.Buffer:sub(0,self.Position-1)..byte..self.Buffer:sub(self.Position+1)
    else
        self.Buffer = self.Buffer..byte
    end
end

function serializer:write_u16(int16, insert, reverse)
	local l1 = bit.rshift(int16, 8)
	local l2 = int16 - bit.lshift(l1, 8)
	l1,l2 = self:check_bytes(l1,l2)
	
	if not(reverse) then
    	self:write_u8(l1, insert)
    	self:write_u8(l2, insert)
	else 
		self:write_u8(l2, insert)
		self:write_u8(l1, insert)
	end
end
 
function serializer:write_u32(int32, insert, reverse)
	local l1 = bit.rshift(int32, 24)
	local l2 = bit.rshift(int32, 16) - bit.lshift(l1, 8)
	local l3 = bit.rshift(int32, 8) - bit.lshift(l1, 16) - bit.lshift(l2, 8)
	local l4 = int32 - bit.lshift(l1, 24) - bit.lshift(l2, 16) - bit.lshift(l3, 8)
	
	l1,l2,l3,l4 = self:check_bytes(l1,l2,l3,l4)
	
	if not(reverse) then
		self:write_u8(l4, insert)
		self:write_u8(l3, insert)
		self:write_u8(l2, insert)
		self:write_u8(l1, insert)
	else
		self:write_u8(l1, insert)
		self:write_u8(l2, insert)
		self:write_u8(l3, insert)
		self:write_u8(l4, insert)
	end
end

function serializer:write_u64(int64, insert, reverse)
	print("serializer:write_u64 value is "..int64)

	local l1 = bit.rshift(int64, 56)
	local l2 = bit.rshift(int64, 48) - bit.lshift(l1, 8)
	local l3 = bit.rshift(int64, 40) - bit.lshift(l1, 16) - bit.lshift(l2, 8)
	local l4 = bit.rshift(int64, 32) - bit.lshift(l1, 24) - bit.lshift(l2, 16) - bit.lshift(l3, 8)
	local l5 = bit.rshift(int64, 24) - bit.lshift(l1, 32) - bit.lshift(l2, 24) - bit.lshift(l3, 16) - bit.lshift(l4, 8)
	local l6 = bit.rshift(int64, 16) - bit.lshift(l1, 40) - bit.lshift(l2, 32) - bit.lshift(l3, 24) - bit.lshift(l4, 16) - bit.lshift(l5, 8)
	local l7 = bit.rshift(int64,  8) - bit.lshift(l1, 48) - bit.lshift(l2, 40) - bit.lshift(l3, 32) - bit.lshift(l4, 24) - bit.lshift(l5, 16) - bit.lshift(l6, 8)
	local l8 = int64 - bit.lshift(l1, 56) - bit.lshift(l2, 48) - bit.lshift(l3, 40) - bit.lshift(l4, 32) - bit.lshift(l5, 24) - bit.lshift(l6, 16) - bit.lshift(l7, 8)

	l1,l2,l3,l4,l5,l6,l7,l8 = self:check_bytes(l1,l2,l3,l4,l5,l6,l7,l8)
	
	if not(reverse) then
		self:write_u8(l8, insert)
		self:write_u8(l7, insert)
		self:write_u8(l6, insert)
		self:write_u8(l5, insert)
		self:write_u8(l4, insert)
		self:write_u8(l3, insert)
		self:write_u8(l2, insert)
		self:write_u8(l1, insert)
	else
		self:write_u8(l1, insert)
		self:write_u8(l2, insert)
		self:write_u8(l3, insert)
		self:write_u8(l4, insert)
		self:write_u8(l5, insert)
		self:write_u8(l6, insert)
		self:write_u8(l7, insert)
		self:write_u8(l8, insert)
	end
end

function serializer:write_string(str, reverse)
    local len = str:len()
	self:write_u16(len, false, reverse)
	self.Buffer = self.Buffer..str
end

return serializer