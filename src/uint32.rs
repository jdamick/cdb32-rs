use std::u32;

pub(crate) fn unpack(data: &[u8]) -> u32 {
    u32::from_le_bytes(data.try_into().unwrap())
}

pub(crate) fn unpack2(buf: &[u8]) -> (u32, u32) {
    assert!(buf.len() >= 8);
    (unpack(&buf[0..4]), unpack(&buf[4..8]))
}

pub(crate) fn pack(data: &mut [u8], src: u32) {
    data[..4].copy_from_slice(&src.to_le_bytes());
}

pub(crate) fn pack2(data: &mut [u8], src0: u32, src1: u32) {
    assert!(data.len() >= 8);
    pack(&mut data[0..4], src0);
    pack(&mut data[4..8], src1);
}

#[test]
fn test_unpack() {
    let data = [0x01, 0x00, 0x00, 0x00];
    assert_eq!(unpack(&data), 1);
}

#[test]
fn test_pack() {
    let mut data = [0; 4];
    pack(&mut data, 1);
    assert_eq!(data, [0x01, 0x00, 0x00, 0x00]);
}

#[test]
fn test_unpack2() {
    let data = [0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00];
    assert_eq!(unpack2(&data), (1, 2));
}

#[test]
fn test_pack2() {
    let mut data = [0; 8];
    pack2(&mut data, 1, 2);
    assert_eq!(data, [0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00]);
}

#[test]
fn test_pack2_overflow() {
    let data = [0; 7];
    assert!(std::panic::catch_unwind(|| {
        let mut data_copy = data;
        pack2(&mut data_copy, 1, 2)
    })
    .is_err());
}

#[test]
fn test_unpack2_overflow() {
    let data = [0; 7];
    assert!(std::panic::catch_unwind(|| {
        let _ = unpack2(&data);
    })
    .is_err());
}
