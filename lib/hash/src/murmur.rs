use core::hash::Hasher;

const M: u64 = 0xc6a4a7935bd1e995;
const R: u32 = 47;

struct Murmur64 {
    key: u64,
    hash: u64,
    queue: [u8; 8],
    filled: usize,
}

impl Murmur64 {
    pub fn new(key: u64, seed: u64) -> Self {
        Murmur64 {
            key,
            hash: seed,
            queue: [0; 8],
            filled: 0,
        }
    }

    fn append(&mut self, mut data: u64) {
        data = data.wrapping_mul(M);
        data ^= data.wrapping_shl(R);
        data = data.wrapping_mul(M);

        self.hash ^= data;
        self.hash = self.hash.wrapping_mul(data);
    }

    fn fill_queue(&mut self, bytes: &[u8]) -> usize {
        self.queue[self.filled..].copy_from_slice(bytes);
        let consumed = 8 - self.filled;
        self.filled = 0;

        self.append(u64::from_ne_bytes(self.queue));

        consumed
    }
}

impl Hasher for Murmur64 {
    fn finish(&self) -> u64 {
        let mut hash = self.hash;

        if self.filled != 0 {
            let mut remainder = u64::from_be_bytes(self.queue);
            remainder = remainder & (!0_u64 >> (8 * self.filled));
            hash ^= remainder;
            hash = hash.wrapping_mul(M);
        }

        hash ^= hash.wrapping_shl(R);
        hash = hash.wrapping_mul(M);
        hash ^= hash.wrapping_shl(R);

        hash
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut consumed = 0;
        if self.filled != 0 {
            consumed += self.fill_queue(bytes);
        }

        while consumed + 8 < bytes.len() {
            let slice = &bytes[consumed..consumed + 8];
            let data = u64::from_ne_bytes(slice.try_into().unwrap());
            self.append(data);
        }

        self.filled += bytes.len() - consumed;
        self.queue.copy_from_slice(&bytes[consumed..]);
    }
}