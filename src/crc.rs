
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for b in data {
        crc = crc ^ ((*b as u16) << 8);

        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc = crc << 1;
            }
        }
    }
    return crc
}

