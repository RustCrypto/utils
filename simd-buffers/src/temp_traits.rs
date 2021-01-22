//! Traits that probably shouldn't live in this crate, but are here temporarily
//! because it's a convenient place for prototyping.

use crate::SimdBuffer;

pub trait BlockEncryptPar<B: SimdBuffer> {
    fn encrypt_par(&self, buffer: &mut B);
}

pub trait StreamCipherPar<B: SimdBuffer> {
    fn try_apply_keystream_par(&mut self, buffer: &mut B);
}

pub trait UniversalHashPar<B: SimdBuffer> {
    fn update_par(&mut self, blocks: &B);
}
