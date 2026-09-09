use crate::types::*;
#[inline]
fn rotr_32(num: u32, amount: c_uint) -> u32 {
    num >> amount | num << 32u32.wrapping_sub(amount)
}
#[inline]
unsafe fn read_be32_word(data: *const u32, index: usize) -> u32 {
    let data = (data as *const u8).add(index * 4);
    u32::from_be_bytes([*data, *data.add(1), *data.add(2), *data.add(3)])
}
static SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0xfc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x6ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];
unsafe fn transform(state: *mut u32, data: *const u32) {
    let mut W: [u32; 16] = [0; 16];
    let mut T: [u32; 8] = [0; 8];
    core::ptr::copy_nonoverlapping(
        state as *const u8,
        ::core::ptr::addr_of_mut!(T) as *mut u8,
        core::mem::size_of::<[u32; 8]>(),
    );
    W[0] = read_be32_word(data, 0);
    T[7] = T[7].wrapping_add(
        rotr_32(T[4] ^ rotr_32(T[4] ^ rotr_32(T[4], 14), 5), 6)
            .wrapping_add(T[6] ^ T[4] & (T[5] ^ T[6]))
            .wrapping_add(SHA256_K[0])
            .wrapping_add(W[0]),
    );
    T[3] = T[3].wrapping_add(T[7]);
    T[7] = T[7].wrapping_add(
        rotr_32(T[0] ^ rotr_32(T[0] ^ rotr_32(T[0], 9), 11), 2)
            .wrapping_add((T[0] & (T[1] ^ T[2])).wrapping_add(T[1] & T[2])),
    );
    W[1] = read_be32_word(data, 1);
    T[6] = T[6].wrapping_add(
        rotr_32(T[3] ^ rotr_32(T[3] ^ rotr_32(T[3], 14), 5), 6)
            .wrapping_add(T[5] ^ T[3] & (T[4] ^ T[5]))
            .wrapping_add(SHA256_K[1])
            .wrapping_add(W[1]),
    );
    T[2] = T[2].wrapping_add(T[6]);
    T[6] = T[6].wrapping_add(
        rotr_32(T[7] ^ rotr_32(T[7] ^ rotr_32(T[7], 9), 11), 2)
            .wrapping_add((T[7] & (T[0] ^ T[1])).wrapping_add(T[0] & T[1])),
    );
    W[2] = read_be32_word(data, 2);
    T[5] = T[5].wrapping_add(
        rotr_32(T[2] ^ rotr_32(T[2] ^ rotr_32(T[2], 14), 5), 6)
            .wrapping_add(T[4] ^ T[2] & (T[3] ^ T[4]))
            .wrapping_add(SHA256_K[2])
            .wrapping_add(W[2]),
    );
    T[1] = T[1].wrapping_add(T[5]);
    T[5] = T[5].wrapping_add(
        rotr_32(T[6] ^ rotr_32(T[6] ^ rotr_32(T[6], 9), 11), 2)
            .wrapping_add((T[6] & (T[7] ^ T[0])).wrapping_add(T[7] & T[0])),
    );
    W[3] = read_be32_word(data, 3);
    T[4] = T[4].wrapping_add(
        rotr_32(T[1] ^ rotr_32(T[1] ^ rotr_32(T[1], 14), 5), 6)
            .wrapping_add(T[3] ^ T[1] & (T[2] ^ T[3]))
            .wrapping_add(SHA256_K[3])
            .wrapping_add(W[3]),
    );
    T[0] = T[0].wrapping_add(T[4]);
    T[4] = T[4].wrapping_add(
        rotr_32(T[5] ^ rotr_32(T[5] ^ rotr_32(T[5], 9), 11), 2)
            .wrapping_add((T[5] & (T[6] ^ T[7])).wrapping_add(T[6] & T[7])),
    );
    W[4] = read_be32_word(data, 4);
    T[3] = T[3].wrapping_add(
        rotr_32(T[0] ^ rotr_32(T[0] ^ rotr_32(T[0], 14), 5), 6)
            .wrapping_add(T[2] ^ T[0] & (T[1] ^ T[2]))
            .wrapping_add(SHA256_K[4])
            .wrapping_add(W[4]),
    );
    T[7] = T[7].wrapping_add(T[3]);
    T[3] = T[3].wrapping_add(
        rotr_32(T[4] ^ rotr_32(T[4] ^ rotr_32(T[4], 9), 11), 2)
            .wrapping_add((T[4] & (T[5] ^ T[6])).wrapping_add(T[5] & T[6])),
    );
    W[5] = read_be32_word(data, 5);
    T[2] = T[2].wrapping_add(
        rotr_32(T[7] ^ rotr_32(T[7] ^ rotr_32(T[7], 14), 5), 6)
            .wrapping_add(T[1] ^ T[7] & (T[0] ^ T[1]))
            .wrapping_add(SHA256_K[5])
            .wrapping_add(W[5]),
    );
    T[6] = T[6].wrapping_add(T[2]);
    T[2] = T[2].wrapping_add(
        rotr_32(T[3] ^ rotr_32(T[3] ^ rotr_32(T[3], 9), 11), 2)
            .wrapping_add((T[3] & (T[4] ^ T[5])).wrapping_add(T[4] & T[5])),
    );
    W[6] = read_be32_word(data, 6);
    T[1] = T[1].wrapping_add(
        rotr_32(T[6] ^ rotr_32(T[6] ^ rotr_32(T[6], 14), 5), 6)
            .wrapping_add(T[0] ^ T[6] & (T[7] ^ T[0]))
            .wrapping_add(SHA256_K[6])
            .wrapping_add(W[6]),
    );
    T[5] = T[5].wrapping_add(T[1]);
    T[1] = T[1].wrapping_add(
        rotr_32(T[2] ^ rotr_32(T[2] ^ rotr_32(T[2], 9), 11), 2)
            .wrapping_add((T[2] & (T[3] ^ T[4])).wrapping_add(T[3] & T[4])),
    );
    W[7] = read_be32_word(data, 7);
    T[0] = T[0].wrapping_add(
        rotr_32(T[5] ^ rotr_32(T[5] ^ rotr_32(T[5], 14), 5), 6)
            .wrapping_add(T[7] ^ T[5] & (T[6] ^ T[7]))
            .wrapping_add(SHA256_K[7])
            .wrapping_add(W[7]),
    );
    T[4] = T[4].wrapping_add(T[0]);
    T[0] = T[0].wrapping_add(
        rotr_32(T[1] ^ rotr_32(T[1] ^ rotr_32(T[1], 9), 11), 2)
            .wrapping_add((T[1] & (T[2] ^ T[3])).wrapping_add(T[2] & T[3])),
    );
    W[8] = read_be32_word(data, 8);
    T[7] = T[7].wrapping_add(
        rotr_32(T[4] ^ rotr_32(T[4] ^ rotr_32(T[4], 14), 5), 6)
            .wrapping_add(T[6] ^ T[4] & (T[5] ^ T[6]))
            .wrapping_add(SHA256_K[8])
            .wrapping_add(W[8]),
    );
    T[3] = T[3].wrapping_add(T[7]);
    T[7] = T[7].wrapping_add(
        rotr_32(T[0] ^ rotr_32(T[0] ^ rotr_32(T[0], 9), 11), 2)
            .wrapping_add((T[0] & (T[1] ^ T[2])).wrapping_add(T[1] & T[2])),
    );
    W[9] = read_be32_word(data, 9);
    T[6] = T[6].wrapping_add(
        rotr_32(T[3] ^ rotr_32(T[3] ^ rotr_32(T[3], 14), 5), 6)
            .wrapping_add(T[5] ^ T[3] & (T[4] ^ T[5]))
            .wrapping_add(SHA256_K[9])
            .wrapping_add(W[9]),
    );
    T[2] = T[2].wrapping_add(T[6]);
    T[6] = T[6].wrapping_add(
        rotr_32(T[7] ^ rotr_32(T[7] ^ rotr_32(T[7], 9), 11), 2)
            .wrapping_add((T[7] & (T[0] ^ T[1])).wrapping_add(T[0] & T[1])),
    );
    W[10] = read_be32_word(data, 10);
    T[5] = T[5].wrapping_add(
        rotr_32(T[2] ^ rotr_32(T[2] ^ rotr_32(T[2], 14), 5), 6)
            .wrapping_add(T[4] ^ T[2] & (T[3] ^ T[4]))
            .wrapping_add(SHA256_K[10])
            .wrapping_add(W[10]),
    );
    T[1] = T[1].wrapping_add(T[5]);
    T[5] = T[5].wrapping_add(
        rotr_32(T[6] ^ rotr_32(T[6] ^ rotr_32(T[6], 9), 11), 2)
            .wrapping_add((T[6] & (T[7] ^ T[0])).wrapping_add(T[7] & T[0])),
    );
    W[11] = read_be32_word(data, 11);
    T[4] = T[4].wrapping_add(
        rotr_32(T[1] ^ rotr_32(T[1] ^ rotr_32(T[1], 14), 5), 6)
            .wrapping_add(T[3] ^ T[1] & (T[2] ^ T[3]))
            .wrapping_add(SHA256_K[11])
            .wrapping_add(W[11]),
    );
    T[0] = T[0].wrapping_add(T[4]);
    T[4] = T[4].wrapping_add(
        rotr_32(T[5] ^ rotr_32(T[5] ^ rotr_32(T[5], 9), 11), 2)
            .wrapping_add((T[5] & (T[6] ^ T[7])).wrapping_add(T[6] & T[7])),
    );
    W[12] = read_be32_word(data, 12);
    T[3] = T[3].wrapping_add(
        rotr_32(T[0] ^ rotr_32(T[0] ^ rotr_32(T[0], 14), 5), 6)
            .wrapping_add(T[2] ^ T[0] & (T[1] ^ T[2]))
            .wrapping_add(SHA256_K[12])
            .wrapping_add(W[12]),
    );
    T[7] = T[7].wrapping_add(T[3]);
    T[3] = T[3].wrapping_add(
        rotr_32(T[4] ^ rotr_32(T[4] ^ rotr_32(T[4], 9), 11), 2)
            .wrapping_add((T[4] & (T[5] ^ T[6])).wrapping_add(T[5] & T[6])),
    );
    W[13] = read_be32_word(data, 13);
    T[2] = T[2].wrapping_add(
        rotr_32(T[7] ^ rotr_32(T[7] ^ rotr_32(T[7], 14), 5), 6)
            .wrapping_add(T[1] ^ T[7] & (T[0] ^ T[1]))
            .wrapping_add(SHA256_K[13])
            .wrapping_add(W[13]),
    );
    T[6] = T[6].wrapping_add(T[2]);
    T[2] = T[2].wrapping_add(
        rotr_32(T[3] ^ rotr_32(T[3] ^ rotr_32(T[3], 9), 11), 2)
            .wrapping_add((T[3] & (T[4] ^ T[5])).wrapping_add(T[4] & T[5])),
    );
    W[14] = read_be32_word(data, 14);
    T[1] = T[1].wrapping_add(
        rotr_32(T[6] ^ rotr_32(T[6] ^ rotr_32(T[6], 14), 5), 6)
            .wrapping_add(T[0] ^ T[6] & (T[7] ^ T[0]))
            .wrapping_add(SHA256_K[14])
            .wrapping_add(W[14]),
    );
    T[5] = T[5].wrapping_add(T[1]);
    T[1] = T[1].wrapping_add(
        rotr_32(T[2] ^ rotr_32(T[2] ^ rotr_32(T[2], 9), 11), 2)
            .wrapping_add((T[2] & (T[3] ^ T[4])).wrapping_add(T[3] & T[4])),
    );
    W[15] = read_be32_word(data, 15);
    T[0] = T[0].wrapping_add(
        rotr_32(T[5] ^ rotr_32(T[5] ^ rotr_32(T[5], 14), 5), 6)
            .wrapping_add(T[7] ^ T[5] & (T[6] ^ T[7]))
            .wrapping_add(SHA256_K[15])
            .wrapping_add(W[15]),
    );
    T[4] = T[4].wrapping_add(T[0]);
    T[0] = T[0].wrapping_add(
        rotr_32(T[1] ^ rotr_32(T[1] ^ rotr_32(T[1], 9), 11), 2)
            .wrapping_add((T[1] & (T[2] ^ T[3])).wrapping_add(T[2] & T[3])),
    );
    let mut j: c_uint = 16;
    while j < 64 {
        W[0] = W[0].wrapping_add(
            (rotr_32(W[14] ^ rotr_32(W[14], 2), 17) ^ W[14] >> 10)
                .wrapping_add(W[9])
                .wrapping_add(rotr_32(W[1] ^ rotr_32(W[1], 11), 7) ^ W[1] >> 3),
        );
        T[7] = T[7].wrapping_add(
            rotr_32(T[4] ^ rotr_32(T[4] ^ rotr_32(T[4], 14), 5), 6)
                .wrapping_add(T[6] ^ T[4] & (T[5] ^ T[6]))
                .wrapping_add(SHA256_K[0u32.wrapping_add(j) as usize])
                .wrapping_add(W[0]),
        );
        T[3] = T[3].wrapping_add(T[7]);
        T[7] = T[7].wrapping_add(
            rotr_32(T[0] ^ rotr_32(T[0] ^ rotr_32(T[0], 9), 11), 2)
                .wrapping_add((T[0] & (T[1] ^ T[2])).wrapping_add(T[1] & T[2])),
        );
        W[1] = W[1].wrapping_add(
            (rotr_32(W[15] ^ rotr_32(W[15], 2), 17) ^ W[15] >> 10)
                .wrapping_add(W[10])
                .wrapping_add(rotr_32(W[2] ^ rotr_32(W[2], 11), 7) ^ W[2] >> 3),
        );
        T[6] = T[6].wrapping_add(
            rotr_32(T[3] ^ rotr_32(T[3] ^ rotr_32(T[3], 14), 5), 6)
                .wrapping_add(T[5] ^ T[3] & (T[4] ^ T[5]))
                .wrapping_add(SHA256_K[1u32.wrapping_add(j) as usize])
                .wrapping_add(W[1]),
        );
        T[2] = T[2].wrapping_add(T[6]);
        T[6] = T[6].wrapping_add(
            rotr_32(T[7] ^ rotr_32(T[7] ^ rotr_32(T[7], 9), 11), 2)
                .wrapping_add((T[7] & (T[0] ^ T[1])).wrapping_add(T[0] & T[1])),
        );
        W[2] = W[2].wrapping_add(
            (rotr_32(W[0] ^ rotr_32(W[0], 2), 17) ^ W[0] >> 10)
                .wrapping_add(W[11])
                .wrapping_add(rotr_32(W[3] ^ rotr_32(W[3], 11), 7) ^ W[3] >> 3),
        );
        T[5] = T[5].wrapping_add(
            rotr_32(T[2] ^ rotr_32(T[2] ^ rotr_32(T[2], 14), 5), 6)
                .wrapping_add(T[4] ^ T[2] & (T[3] ^ T[4]))
                .wrapping_add(SHA256_K[2u32.wrapping_add(j) as usize])
                .wrapping_add(W[2]),
        );
        T[1] = T[1].wrapping_add(T[5]);
        T[5] = T[5].wrapping_add(
            rotr_32(T[6] ^ rotr_32(T[6] ^ rotr_32(T[6], 9), 11), 2)
                .wrapping_add((T[6] & (T[7] ^ T[0])).wrapping_add(T[7] & T[0])),
        );
        W[3] = W[3].wrapping_add(
            (rotr_32(W[1] ^ rotr_32(W[1], 2), 17) ^ W[1] >> 10)
                .wrapping_add(W[12])
                .wrapping_add(rotr_32(W[4] ^ rotr_32(W[4], 11), 7) ^ W[4] >> 3),
        );
        T[4] = T[4].wrapping_add(
            rotr_32(T[1] ^ rotr_32(T[1] ^ rotr_32(T[1], 14), 5), 6)
                .wrapping_add(T[3] ^ T[1] & (T[2] ^ T[3]))
                .wrapping_add(SHA256_K[3u32.wrapping_add(j) as usize])
                .wrapping_add(W[3]),
        );
        T[0] = T[0].wrapping_add(T[4]);
        T[4] = T[4].wrapping_add(
            rotr_32(T[5] ^ rotr_32(T[5] ^ rotr_32(T[5], 9), 11), 2)
                .wrapping_add((T[5] & (T[6] ^ T[7])).wrapping_add(T[6] & T[7])),
        );
        W[4] = W[4].wrapping_add(
            (rotr_32(W[2] ^ rotr_32(W[2], 2), 17) ^ W[2] >> 10)
                .wrapping_add(W[13])
                .wrapping_add(rotr_32(W[5] ^ rotr_32(W[5], 11), 7) ^ W[5] >> 3),
        );
        T[3] = T[3].wrapping_add(
            rotr_32(T[0] ^ rotr_32(T[0] ^ rotr_32(T[0], 14), 5), 6)
                .wrapping_add(T[2] ^ T[0] & (T[1] ^ T[2]))
                .wrapping_add(SHA256_K[4u32.wrapping_add(j) as usize])
                .wrapping_add(W[4]),
        );
        T[7] = T[7].wrapping_add(T[3]);
        T[3] = T[3].wrapping_add(
            rotr_32(T[4] ^ rotr_32(T[4] ^ rotr_32(T[4], 9), 11), 2)
                .wrapping_add((T[4] & (T[5] ^ T[6])).wrapping_add(T[5] & T[6])),
        );
        W[5] = W[5].wrapping_add(
            (rotr_32(W[3] ^ rotr_32(W[3], 2), 17) ^ W[3] >> 10)
                .wrapping_add(W[14])
                .wrapping_add(rotr_32(W[6] ^ rotr_32(W[6], 11), 7) ^ W[6] >> 3),
        );
        T[2] = T[2].wrapping_add(
            rotr_32(T[7] ^ rotr_32(T[7] ^ rotr_32(T[7], 14), 5), 6)
                .wrapping_add(T[1] ^ T[7] & (T[0] ^ T[1]))
                .wrapping_add(SHA256_K[5u32.wrapping_add(j) as usize])
                .wrapping_add(W[5]),
        );
        T[6] = T[6].wrapping_add(T[2]);
        T[2] = T[2].wrapping_add(
            rotr_32(T[3] ^ rotr_32(T[3] ^ rotr_32(T[3], 9), 11), 2)
                .wrapping_add((T[3] & (T[4] ^ T[5])).wrapping_add(T[4] & T[5])),
        );
        W[6] = W[6].wrapping_add(
            (rotr_32(W[4] ^ rotr_32(W[4], 2), 17) ^ W[4] >> 10)
                .wrapping_add(W[15])
                .wrapping_add(rotr_32(W[7] ^ rotr_32(W[7], 11), 7) ^ W[7] >> 3),
        );
        T[1] = T[1].wrapping_add(
            rotr_32(T[6] ^ rotr_32(T[6] ^ rotr_32(T[6], 14), 5), 6)
                .wrapping_add(T[0] ^ T[6] & (T[7] ^ T[0]))
                .wrapping_add(SHA256_K[6u32.wrapping_add(j) as usize])
                .wrapping_add(W[6]),
        );
        T[5] = T[5].wrapping_add(T[1]);
        T[1] = T[1].wrapping_add(
            rotr_32(T[2] ^ rotr_32(T[2] ^ rotr_32(T[2], 9), 11), 2)
                .wrapping_add((T[2] & (T[3] ^ T[4])).wrapping_add(T[3] & T[4])),
        );
        W[7] = W[7].wrapping_add(
            (rotr_32(W[5] ^ rotr_32(W[5], 2), 17) ^ W[5] >> 10)
                .wrapping_add(W[0])
                .wrapping_add(rotr_32(W[8] ^ rotr_32(W[8], 11), 7) ^ W[8] >> 3),
        );
        T[0] = T[0].wrapping_add(
            rotr_32(T[5] ^ rotr_32(T[5] ^ rotr_32(T[5], 14), 5), 6)
                .wrapping_add(T[7] ^ T[5] & (T[6] ^ T[7]))
                .wrapping_add(SHA256_K[7u32.wrapping_add(j) as usize])
                .wrapping_add(W[7]),
        );
        T[4] = T[4].wrapping_add(T[0]);
        T[0] = T[0].wrapping_add(
            rotr_32(T[1] ^ rotr_32(T[1] ^ rotr_32(T[1], 9), 11), 2)
                .wrapping_add((T[1] & (T[2] ^ T[3])).wrapping_add(T[2] & T[3])),
        );
        W[8] = W[8].wrapping_add(
            (rotr_32(W[6] ^ rotr_32(W[6], 2), 17) ^ W[6] >> 10)
                .wrapping_add(W[1])
                .wrapping_add(rotr_32(W[9] ^ rotr_32(W[9], 11), 7) ^ W[9] >> 3),
        );
        T[7] = T[7].wrapping_add(
            rotr_32(T[4] ^ rotr_32(T[4] ^ rotr_32(T[4], 14), 5), 6)
                .wrapping_add(T[6] ^ T[4] & (T[5] ^ T[6]))
                .wrapping_add(SHA256_K[8u32.wrapping_add(j) as usize])
                .wrapping_add(W[8]),
        );
        T[3] = T[3].wrapping_add(T[7]);
        T[7] = T[7].wrapping_add(
            rotr_32(T[0] ^ rotr_32(T[0] ^ rotr_32(T[0], 9), 11), 2)
                .wrapping_add((T[0] & (T[1] ^ T[2])).wrapping_add(T[1] & T[2])),
        );
        W[9] = W[9].wrapping_add(
            (rotr_32(W[7] ^ rotr_32(W[7], 2), 17) ^ W[7] >> 10)
                .wrapping_add(W[2])
                .wrapping_add(rotr_32(W[10] ^ rotr_32(W[10], 11), 7) ^ W[10] >> 3),
        );
        T[6] = T[6].wrapping_add(
            rotr_32(T[3] ^ rotr_32(T[3] ^ rotr_32(T[3], 14), 5), 6)
                .wrapping_add(T[5] ^ T[3] & (T[4] ^ T[5]))
                .wrapping_add(SHA256_K[9u32.wrapping_add(j) as usize])
                .wrapping_add(W[9]),
        );
        T[2] = T[2].wrapping_add(T[6]);
        T[6] = T[6].wrapping_add(
            rotr_32(T[7] ^ rotr_32(T[7] ^ rotr_32(T[7], 9), 11), 2)
                .wrapping_add((T[7] & (T[0] ^ T[1])).wrapping_add(T[0] & T[1])),
        );
        W[10] = W[10].wrapping_add(
            (rotr_32(W[8] ^ rotr_32(W[8], 2), 17) ^ W[8] >> 10)
                .wrapping_add(W[3])
                .wrapping_add(rotr_32(W[11] ^ rotr_32(W[11], 11), 7) ^ W[11] >> 3),
        );
        T[5] = T[5].wrapping_add(
            rotr_32(T[2] ^ rotr_32(T[2] ^ rotr_32(T[2], 14), 5), 6)
                .wrapping_add(T[4] ^ T[2] & (T[3] ^ T[4]))
                .wrapping_add(SHA256_K[10u32.wrapping_add(j) as usize])
                .wrapping_add(W[10]),
        );
        T[1] = T[1].wrapping_add(T[5]);
        T[5] = T[5].wrapping_add(
            rotr_32(T[6] ^ rotr_32(T[6] ^ rotr_32(T[6], 9), 11), 2)
                .wrapping_add((T[6] & (T[7] ^ T[0])).wrapping_add(T[7] & T[0])),
        );
        W[11] = W[11].wrapping_add(
            (rotr_32(W[9] ^ rotr_32(W[9], 2), 17) ^ W[9] >> 10)
                .wrapping_add(W[4])
                .wrapping_add(rotr_32(W[12] ^ rotr_32(W[12], 11), 7) ^ W[12] >> 3),
        );
        T[4] = T[4].wrapping_add(
            rotr_32(T[1] ^ rotr_32(T[1] ^ rotr_32(T[1], 14), 5), 6)
                .wrapping_add(T[3] ^ T[1] & (T[2] ^ T[3]))
                .wrapping_add(SHA256_K[11u32.wrapping_add(j) as usize])
                .wrapping_add(W[11]),
        );
        T[0] = T[0].wrapping_add(T[4]);
        T[4] = T[4].wrapping_add(
            rotr_32(T[5] ^ rotr_32(T[5] ^ rotr_32(T[5], 9), 11), 2)
                .wrapping_add((T[5] & (T[6] ^ T[7])).wrapping_add(T[6] & T[7])),
        );
        W[12] = W[12].wrapping_add(
            (rotr_32(W[10] ^ rotr_32(W[10], 2), 17) ^ W[10] >> 10)
                .wrapping_add(W[5])
                .wrapping_add(rotr_32(W[13] ^ rotr_32(W[13], 11), 7) ^ W[13] >> 3),
        );
        T[3] = T[3].wrapping_add(
            rotr_32(T[0] ^ rotr_32(T[0] ^ rotr_32(T[0], 14), 5), 6)
                .wrapping_add(T[2] ^ T[0] & (T[1] ^ T[2]))
                .wrapping_add(SHA256_K[12u32.wrapping_add(j) as usize])
                .wrapping_add(W[12]),
        );
        T[7] = T[7].wrapping_add(T[3]);
        T[3] = T[3].wrapping_add(
            rotr_32(T[4] ^ rotr_32(T[4] ^ rotr_32(T[4], 9), 11), 2)
                .wrapping_add((T[4] & (T[5] ^ T[6])).wrapping_add(T[5] & T[6])),
        );
        W[13] = W[13].wrapping_add(
            (rotr_32(W[11] ^ rotr_32(W[11], 2), 17) ^ W[11] >> 10)
                .wrapping_add(W[6])
                .wrapping_add(rotr_32(W[14] ^ rotr_32(W[14], 11), 7) ^ W[14] >> 3),
        );
        T[2] = T[2].wrapping_add(
            rotr_32(T[7] ^ rotr_32(T[7] ^ rotr_32(T[7], 14), 5), 6)
                .wrapping_add(T[1] ^ T[7] & (T[0] ^ T[1]))
                .wrapping_add(SHA256_K[13u32.wrapping_add(j) as usize])
                .wrapping_add(W[13]),
        );
        T[6] = T[6].wrapping_add(T[2]);
        T[2] = T[2].wrapping_add(
            rotr_32(T[3] ^ rotr_32(T[3] ^ rotr_32(T[3], 9), 11), 2)
                .wrapping_add((T[3] & (T[4] ^ T[5])).wrapping_add(T[4] & T[5])),
        );
        W[14] = W[14].wrapping_add(
            (rotr_32(W[12] ^ rotr_32(W[12], 2), 17) ^ W[12] >> 10)
                .wrapping_add(W[7])
                .wrapping_add(rotr_32(W[15] ^ rotr_32(W[15], 11), 7) ^ W[15] >> 3),
        );
        T[1] = T[1].wrapping_add(
            rotr_32(T[6] ^ rotr_32(T[6] ^ rotr_32(T[6], 14), 5), 6)
                .wrapping_add(T[0] ^ T[6] & (T[7] ^ T[0]))
                .wrapping_add(SHA256_K[14u32.wrapping_add(j) as usize])
                .wrapping_add(W[14]),
        );
        T[5] = T[5].wrapping_add(T[1]);
        T[1] = T[1].wrapping_add(
            rotr_32(T[2] ^ rotr_32(T[2] ^ rotr_32(T[2], 9), 11), 2)
                .wrapping_add((T[2] & (T[3] ^ T[4])).wrapping_add(T[3] & T[4])),
        );
        W[15] = W[15].wrapping_add(
            (rotr_32(W[13] ^ rotr_32(W[13], 2), 17) ^ W[13] >> 10)
                .wrapping_add(W[8])
                .wrapping_add(rotr_32(W[0] ^ rotr_32(W[0], 11), 7) ^ W[0] >> 3),
        );
        T[0] = T[0].wrapping_add(
            rotr_32(T[5] ^ rotr_32(T[5] ^ rotr_32(T[5], 14), 5), 6)
                .wrapping_add(T[7] ^ T[5] & (T[6] ^ T[7]))
                .wrapping_add(SHA256_K[15u32.wrapping_add(j) as usize])
                .wrapping_add(W[15]),
        );
        T[4] = T[4].wrapping_add(T[0]);
        T[0] = T[0].wrapping_add(
            rotr_32(T[1] ^ rotr_32(T[1] ^ rotr_32(T[1], 9), 11), 2)
                .wrapping_add((T[1] & (T[2] ^ T[3])).wrapping_add(T[2] & T[3])),
        );
        j = j.wrapping_add(16);
    }
    for i in 0..8 {
        *state.add(i) = (*state.add(i)).wrapping_add(T[i]);
    }
}
unsafe fn process(check: *mut lzma_check_state) {
    transform(
        ::core::ptr::addr_of_mut!((*check).state.sha256.state) as *mut u32,
        ::core::ptr::addr_of_mut!((*check).buffer.u32_0) as *const u32,
    );
}
pub unsafe fn lzma_sha256_init(check: *mut lzma_check_state) {
    let s: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    core::ptr::copy_nonoverlapping(
        ::core::ptr::addr_of!(s) as *const u8,
        ::core::ptr::addr_of_mut!((*check).state.sha256.state) as *mut u8,
        core::mem::size_of::<[u32; 8]>(),
    );
    (*check).state.sha256.size = 0;
}
pub unsafe fn lzma_sha256_update(
    mut buf: *const u8,
    mut size: size_t,
    check: *mut lzma_check_state,
) {
    while size > 0 {
        let copy_start: size_t = ((*check).state.sha256.size & 0x3f as u64) as size_t;
        let mut copy_size: size_t = (64_usize).wrapping_sub(copy_start);
        if copy_size > size {
            copy_size = size;
        }
        core::ptr::copy_nonoverlapping(
            buf as *const u8,
            (::core::ptr::addr_of_mut!((*check).buffer.u8_0) as *mut u8).add(copy_start) as *mut u8,
            copy_size,
        );
        buf = buf.add(copy_size);
        size = size.wrapping_sub(copy_size);
        (*check).state.sha256.size = (*check).state.sha256.size.wrapping_add(copy_size as u64);
        if (*check).state.sha256.size & 0x3f as u64 == 0 {
            process(check);
        }
    }
}
pub unsafe fn lzma_sha256_finish(check: *mut lzma_check_state) {
    let mut pos: size_t = ((*check).state.sha256.size & 0x3f as u64) as size_t;
    (*check).buffer.u8_0[pos as usize] = 0x80 as u8;
    pos += 1;
    while pos != (64 - 8) as size_t {
        if pos == 64 {
            process(check);
            pos = 0;
        }
        (*check).buffer.u8_0[pos as usize] = 0;
        pos += 1;
    }
    (*check).state.sha256.size = (*check).state.sha256.size.wrapping_mul(8);
    let size_bytes = (*check).state.sha256.size.to_be_bytes();
    core::ptr::copy_nonoverlapping(
        size_bytes.as_ptr(),
        (::core::ptr::addr_of_mut!((*check).buffer.u8_0) as *mut u8).add(64 - 8),
        size_bytes.len(),
    );
    process(check);
    let mut i: size_t = 0;
    while i < 8 {
        let bytes = (*check).state.sha256.state[i as usize].to_be_bytes();
        core::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            (::core::ptr::addr_of_mut!((*check).buffer.u8_0) as *mut u8).add(i as usize * 4),
            bytes.len(),
        );
        i += 1;
    }
}
