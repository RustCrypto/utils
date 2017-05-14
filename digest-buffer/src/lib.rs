#![no_std]
extern crate byte_tools;
extern crate generic_array;
use generic_array::{GenericArray, ArrayLength};
use byte_tools::zero;

type Block<N> = GenericArray<u8, N>;

#[derive(Clone, Copy)]
pub struct DigestBuffer<N: ArrayLength<u8>> where N::ArrayType: Copy {
    buffer: GenericArray<u8, N>,
    pos: usize,
}

impl <N: ArrayLength<u8>> DigestBuffer<N> where N::ArrayType: Copy {
    pub fn new() -> DigestBuffer<N> {
        DigestBuffer::<N> {
            buffer: Default::default(),
            pos: 0,
        }
    }

    pub fn input<F: FnMut(&Block<N>)>(&mut self, mut input: &[u8], mut func: F) {
        // If there is already data in the buffer, copy as much as we can
        // into it and process the data if the buffer becomes full.
        if self.pos != 0 {
            let rem = N::to_usize() - self.pos;

            if input.len() >= rem {
                let (l, r) = input.split_at(rem);
                input = r;
                self.buffer[self.pos..].copy_from_slice(l);
                self.pos = 0;
                func(&self.buffer);
            } else {
                let end = self.pos + input.len();
                self.buffer[self.pos..end].copy_from_slice(input);
                self.pos = end;
                return;
            }
        }

        // While we have at least a full buffer size chunks's worth of data,
        // process that data without copying it into the buffer
        while input.len() >= N::to_usize() {
            let (l, r) = input.split_at(N::to_usize());
            input = r;
            let block = GenericArray::from_slice(&l);
            func(block);
        }

        // Copy any input data into the buffer. At this point in the method,
        // the ammount of data left in the input vector will be less than
        // the buffer size and the buffer will be empty.
        self.buffer[..input.len()].copy_from_slice(input);
        self.pos += input.len();
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    pub fn zero_until(&mut self, idx: usize) {
        assert!(idx >= self.pos);
        zero(&mut self.buffer[self.pos..idx]);
        self.pos = idx;
    }

    pub fn next(&mut self, len: usize) -> &mut [u8] {
        self.pos += len;
        &mut self.buffer[self.pos - len..self.pos]
    }

    pub fn full_buffer(& mut self) -> &Block<N> {
        assert!(self.pos == self.size());
        self.pos = 0;
        &self.buffer
    }

    pub fn current_buffer(&mut self) -> &[u8] {
        let tmp = self.pos;
        self.pos = 0;
        &self.buffer[..tmp]
    }

    pub fn position(&self) -> usize { self.pos }

    pub fn remaining(&self) -> usize { self.size() - self.pos }

    pub fn standard_padding<F: FnMut(&Block<N>)>(&mut self, rem: usize, mut func: F) {
        let size = self.size();

        self.next(1)[0] = 128;

        if self.remaining() < rem {
            self.zero_until(size);
            func(self.full_buffer());
        }

        self.zero_until(size - rem);
    }

    pub fn size(&self) -> usize {
         N::to_usize()
    }
}

impl <N: ArrayLength<u8>> Default for DigestBuffer<N> where N::ArrayType: Copy {
    fn default() -> Self { Self::new() }
}
