//! Basic tests
use keccak::{Keccak, State1600};
use sponge_cursor::SpongeCursor;

const DATA_LEN: usize = 500;
const CHUNK_SIZES: &[usize] = &[1, 17, 133, 139, 203, 300];

static SHORT_DATA: [u8; 12] = *b"hello world!";

#[allow(clippy::cast_possible_truncation)]
static DATA: [u8; DATA_LEN] = {
    let mut buf = [0u8; DATA_LEN];
    let mut i = 0;
    while i < buf.len() {
        buf[i] = (i % 217) as u8;
        i += 1;
    }
    buf
};

static CONCAT_DATA: [u8; SHORT_DATA.len() + DATA_LEN] = {
    let mut buf = [0u8; SHORT_DATA.len() + DATA_LEN];
    // TODO(MSRV-1.87): use `copy_from_slice`
    let mut i = 0;
    while i < buf.len() {
        buf[i] = if i < SHORT_DATA.len() {
            SHORT_DATA[i]
        } else {
            DATA[i - SHORT_DATA.len()]
        };
        i += 1;
    }
    buf
};

static STATE1: State1600 = [
    0x9C40D618E6055156,
    0x44755CBBEAEBC6F9,
    0xF903B381412808D8,
    0x0339BEC23E8A90AB,
    0xF32F96B8456CB8AF,
    0xCE6349D647F4BE3C,
    0x1AA37FEB280A570F,
    0x99233F523925282C,
    0x835AA18F6EEA40E5,
    0xAEEFA72F28932C4C,
    0x8A602BA780C75936,
    0x16E389AAA33064F0,
    0xB2889A3A30E2DCC3,
    0x052546C55F62FF34,
    0xD7146414F4796631,
    0x77EE6E84441B567A,
    0xE2C698D4357E57AF,
    0x744AE4F10636A122,
    0x020E03048CD9CD5E,
    0x43C4B8081CF9C548,
    0x81A3AED00DD27357,
    0x54159D6C3AF969DE,
    0xD73C4B34CAB22195,
    0xEDD07D1D23162C0D,
    0x87CBF66555849682,
];

static SQUEEZE_DATA: &[u8; DATA_LEN] = include_bytes!("data/squeeze.bin");

#[test]
fn keccak_test() {
    Keccak::new().with_f1600(|f1600| {
        let mut cursor = SpongeCursor::<136>::default();
        let mut state = State1600::default();

        cursor.absorb_u64_le(&mut state, f1600, &SHORT_DATA);
        assert_eq!(state[0], 0x6F77_206F_6C6C_6568);
        assert_eq!(state[1], 0x0000_0000_2164_6C72);
        assert!(state[2..].iter().all(|v| *v == 0));
        assert_eq!(cursor.pos(), SHORT_DATA.len());
        assert_eq!(usize::from(cursor.raw_pos()), SHORT_DATA.len());
        let short_len = u8::try_from(SHORT_DATA.len()).unwrap();
        assert_eq!(cursor, SpongeCursor::new(short_len).unwrap());

        let expected_pos = (SHORT_DATA.len() + DATA.len()) % 136;
        for &chunk_size in CHUNK_SIZES {
            let mut state_copy = state;
            let mut cursor_copy = cursor.clone();

            for chunk in DATA.chunks(chunk_size) {
                cursor_copy.absorb_u64_le(&mut state_copy, f1600, chunk);
            }
            assert_eq!(state_copy, STATE1);
            assert_eq!(cursor_copy.pos(), expected_pos);
            assert_eq!(usize::from(cursor_copy.raw_pos()), expected_pos);
        }

        {
            let mut cursor = SpongeCursor::<136>::default();
            let mut state = State1600::default();

            cursor.absorb_u64_le(&mut state, f1600, &CONCAT_DATA);
            assert_eq!(state, STATE1);
            assert_eq!(cursor.pos(), expected_pos);
            assert_eq!(usize::from(cursor.raw_pos()), expected_pos);
        }

        cursor.absorb_u64_le(&mut state, f1600, &DATA);
        assert_eq!(state, STATE1);
        assert_eq!(cursor.pos(), expected_pos);
        assert_eq!(usize::from(cursor.raw_pos()), expected_pos);

        f1600(&mut state);

        for &chunk_size in CHUNK_SIZES {
            let mut buf = [0u8; DATA_LEN];
            cursor = Default::default();
            let mut state_copy = state;

            for chunk in buf.chunks_mut(chunk_size) {
                cursor.squeeze_read_u64_le(&mut state_copy, f1600, chunk);
            }

            assert_eq!(&buf, SQUEEZE_DATA);
        }

        let expected: [u8; DATA_LEN] = core::array::from_fn(|i| DATA[i] ^ SQUEEZE_DATA[i]);
        for &chunk_size in CHUNK_SIZES {
            let mut buf = DATA;
            cursor = Default::default();
            let mut state_copy = state;

            for chunk in buf.chunks_mut(chunk_size) {
                cursor.squeeze_xor_u64_le(&mut state_copy, f1600, chunk);
            }

            assert_eq!(buf, expected);
        }
    });
}
