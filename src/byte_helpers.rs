pub struct Buffer<T> {
    items: ~[T],
    mut length: uint,
    mut pos: uint
}

pub pure fn Buffer<T>(items: ~[T]) -> Buffer<T> {
    let len = vec::len(items);
    Buffer {
        items: move items,
        length: len,
        pos: 0
    }
}

impl<T: Copy> Buffer<T> {
    pure fn eof(&self) -> bool {return self.pos == self.length;}
    fn read(&self, number: uint) -> ~[T] {
        assert (number + self.pos <= self.length);
        let return_data = vec::slice(self.items, self.pos, self.pos + number);
        self.pos += number;
        return return_data;
    }

    fn seek(&self, pos: uint) {
        assert pos >= 0 && pos <= self.length;
        self.pos = pos;
    }
}

//TODO: Make me part of the generic buffer!
pub struct ByteBuffer {
    bytes: ~[u8],
    mut length: uint,
    mut pos: uint
}


pub pure fn ByteBuffer(bytes: ~[u8]) -> ByteBuffer {
    let len = vec::len(bytes);
    ByteBuffer {
        bytes: move bytes,
        length: len,
        pos: 0
    }
}

impl ByteBuffer {
    pure fn eof() -> bool {return self.pos == self.length;}
    fn read(number: uint) -> ~[u8] {
        assert (number + self.pos <= self.length);
        let return_data = vec::slice(self.bytes, self.pos, self.pos + number);
        self.pos += number;
        return return_data;
    }

    fn read_le_uint(number: uint) -> uint {
        return bytes_to_le_uint(self.read(number));
    }

    fn seek(pos: uint) {
        assert pos >= 0 && pos <= self.length;
        self.pos = pos;
    }
}

pub fn bytes_to_le_uint(bytes: ~[u8]) -> uint {

    let mut val = 0u, pos = 0u, i = 0;
    let size:uint = vec::len(bytes);

    while i < size {
        val += bytes[i] as uint << pos;
        pos += 8u;
        i += 1;
    }

    return val;
}

pub fn bytes_to_be_uint(bytes: ~[u8]) -> uint {
    let mut val = 0u, i = vec::len(bytes);
    while i > 0u {
        i -= 1u;
        val += (bytes[i] as uint) << i * 8u;
    }
    return val;
}

pub fn uint_to_le_bytes(n: u64, size: uint) -> ~[u8] {
    match size {
        1u => { return ~[n as u8] }
        2u => { return ~[n as u8, (n >> 8) as u8] }
        4u => { return ~[n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8] }
        8u => { return ~[n as u8,
            (n >> 8) as u8,
            (n >> 16) as u8,
            (n >> 24) as u8,
            (n >> 32) as u8,
            (n >> 40) as u8,
            (n >> 48) as u8,
            (n >> 56) as u8] 
        }
        _ => {
            let mut bytes: ~[u8] = ~[], i = size, n = n;
            while i > 0u {
                bytes.push((n & 255_u64) as u8);
                n >>= 8_u64;
                i -= 1u;
            }
            return bytes;
        }
    }
}

pub fn u8vec_to_u16vec(input: ~[u8]) -> ~[u16] {
    let mut output: ~[u16] = ~[];
    let len = vec::len(input);
    let mut i:uint = 0;
    
    assert len % 2 == 0;

    while (i < (len / 2)) {
        output.push((input[i * 2] as u16 + ((input[(i * 2) + 1] as u16) << 8)));
        i += 1;
    }

    return output;
}
