//! Basic tests
use keccak::{Keccak, State1600};
use sponge_cursor::SpongeCursor;

const DATA_LEN: usize = 500;
const CHUNK_SIZES: &[usize] = &[1, 17, 133, 139, 203, 300];

static SHORT_DATA: [u8; 12] = *b"hello world!";

#[allow(clippy::cast_possible_truncation)]
static LONG_DATA: [u8; DATA_LEN] = {
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
            LONG_DATA[i - SHORT_DATA.len()]
        };
        i += 1;
    }
    buf
};

static STATE: State1600 = [
    0xB2970DE4121985B5,
    0x29C223B87687A340,
    0x4A146BF36ABB1CC8,
    0x3D3BCBE920587765,
    0x8293EBA8BC356351,
    0x329C7414B019521A,
    0x5E733DF382609717,
    0x20F3B803705BCC28,
    0xB1A1DCC9DC4FCD20,
    0x084A6874972249D7,
    0x85C9D5D5437F2081,
    0x84E9EDD8482329E0,
    0x2E6259B24480185D,
    0x9B9F2680A9872C6A,
    0xCB08DC6DF5ADBB78,
    0x12BBABF030B4A3DE,
    0x82F72D4928A50AE4,
    0x1D071289E249ACB5,
    0x701BC30DFA4E5DF5,
    0xBA4910413F89A74E,
    0x3CCD8011250F3584,
    0x9548960143837F4C,
    0xC025CF3059BE5E14,
    0x0863BB97DA339D82,
    0x207CEB1D10C336C4,
];

static SQUEEZE_DATA: &[u8; DATA_LEN] = include_bytes!("data/squeeze.bin");

#[test]
fn keccak_test() {
    Keccak::new().with_p1600::<1>(|p1600| {
        let mut cursor = SpongeCursor::<136>::default();
        let mut state = State1600::default();

        cursor.absorb_u64_le(&mut state, p1600, &SHORT_DATA);
        assert_eq!(state[0], 0x6F77_206F_6C6C_6568);
        assert_eq!(state[1], 0x0000_0000_2164_6C72);
        assert!(state[2..].iter().all(|v| *v == 0));
        assert_eq!(cursor.pos(), SHORT_DATA.len());
        assert_eq!(usize::from(cursor.raw_pos()), SHORT_DATA.len());
        let short_len = u8::try_from(SHORT_DATA.len()).unwrap();
        assert_eq!(cursor, SpongeCursor::new(short_len).unwrap());

        let expected_pos = (SHORT_DATA.len() + LONG_DATA.len()) % 136;
        for &chunk_size in CHUNK_SIZES {
            let mut state_copy = state;
            let mut cursor_copy = cursor.clone();

            for chunk in LONG_DATA.chunks(chunk_size) {
                cursor_copy.absorb_u64_le(&mut state_copy, p1600, chunk);
            }
            assert_eq!(state_copy, STATE);
            assert_eq!(cursor_copy.pos(), expected_pos);
            assert_eq!(usize::from(cursor_copy.raw_pos()), expected_pos);
        }

        {
            let mut cursor = SpongeCursor::<136>::default();
            let mut state = State1600::default();

            cursor.absorb_u64_le(&mut state, p1600, &CONCAT_DATA);
            assert_eq!(state, STATE);
            assert_eq!(cursor.pos(), expected_pos);
            assert_eq!(usize::from(cursor.raw_pos()), expected_pos);
        }

        cursor.absorb_u64_le(&mut state, p1600, &LONG_DATA);
        assert_eq!(state, STATE);
        assert_eq!(cursor.pos(), expected_pos);
        assert_eq!(usize::from(cursor.raw_pos()), expected_pos);

        p1600(&mut state);

        for &chunk_size in CHUNK_SIZES {
            let mut buf = [0u8; DATA_LEN];
            cursor = Default::default();
            let mut state_copy = state;

            for chunk in buf.chunks_mut(chunk_size) {
                cursor.squeeze_read_u64_le(&mut state_copy, p1600, chunk);
            }

            assert_eq!(&buf, SQUEEZE_DATA);
        }

        let expected: [u8; DATA_LEN] = core::array::from_fn(|i| LONG_DATA[i] ^ SQUEEZE_DATA[i]);
        for &chunk_size in CHUNK_SIZES {
            let mut buf = LONG_DATA;
            cursor = Default::default();
            let mut state_copy = state;

            for chunk in buf.chunks_mut(chunk_size) {
                cursor.squeeze_xor_u64_le(&mut state_copy, p1600, chunk);
            }

            assert_eq!(buf, expected);
        }
    });
}
