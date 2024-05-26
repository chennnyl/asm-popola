use crate::vm::Devola;

/// Returns a read-only slice of memory.
pub fn memgetn(devola: &mut Devola, start: u16, size: u16) -> &[u8] {
    &devola.memory.memory[start as usize..(start+size) as usize]
}
/// Sets all bytes in the range to 0.
pub fn memclear(devola: &mut Devola, start: u16, size: u16) {
    for i in start..start+size {
        devola.memory[i] = 0;
    }
}
/// Copies all data from the source slice to the destination slice.
pub fn memcpy(devola: &mut Devola, source: u16, destination: u16, size: u16) {
    for i in 0..size {
        devola.memory[destination+i] = devola.memory[source+i];
    }
}
/// Copy bytes from a source buffer
pub fn memset(devola: &mut Devola, source: &[u8], destination: u16, size: u16) {
    for i in 0..size {
        devola.memory[destination+i] = source[i as usize];
    }
}

pub mod interface {
    use super::*;
    use crate::util;
    use std::collections::HashMap;

    /// `memclear(start_hi, start_lo, size_hi, size_lo)`
    ///
    /// Accepts arguments from the stack. Sets the specified range of `size` bytes
    /// starting at `start` in memory to 0.
    pub fn i_memclear(devola: &mut Devola) {
        let (size_lo, size_hi) = (devola.pop(), devola.pop());
        let (start_lo, start_hi) = (devola.pop(), devola.pop());
        let size = util::build_u16(size_hi, size_lo);
        let start = util::build_u16(start_hi, start_lo);
        memclear(devola, start, size);
    }

    /// `memcpy(source_hi, source_lo, dest_hi, dest_lo, size_hi, size_lo)`
    ///
    /// Accepts arguments from the stack. Copies `size` bytes starting from `source` to the
    /// range starting at `dest`.
    pub fn i_memcpy(devola: &mut Devola) {
        let (size_lo, size_hi) = (devola.pop(), devola.pop());
        let (dest_lo, dest_hi) = (devola.pop(), devola.pop());
        let (source_lo, source_hi) = (devola.pop(), devola.pop());
        let size = util::build_u16(size_hi, size_lo);
        let destination = util::build_u16(dest_hi, dest_lo);
        let source = util::build_u16(source_hi, source_lo);
        memcpy(devola, source, destination, size);
    }

    pub fn i_debug_println(devola: &mut Devola) {
        let argc = devola.pop();
        let mut argv: Vec<u8> = Vec::with_capacity(argc as usize);
        argv.fill_with(|| devola.pop());
        let argv: Vec<&u8> = argv.iter().rev().collect();

        println!("{argv:?}");
    }

    pub type DevolaExtern = dyn FnMut(&mut Devola) -> ();
    pub type DevolaExternTable = HashMap<String, Box<DevolaExtern>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memgetn_clear() {
        let mut devola = crate::util::execute_file("sample/stdlib_tests/memgetn.pop").unwrap();
        let range = memgetn(&mut devola, 0, 256);

        assert!((0..=255u8).all(|n| n == range[n as usize]));

        memclear(&mut devola, 0, 256);
        let range = memgetn(&mut devola, 0, 256);
        assert!((0..=255u8).all(|n| range[n as usize] == 0));
    }

    #[test]
    fn test_memset() {
        let mut devola = crate::util::execute_file("sample/stdlib_tests/memgetn.pop").unwrap();
        let buffer: Vec<u8> = vec![3, 14, 1, 5];

        memset(&mut devola, buffer.as_slice(), 0, 4);
        let range = memgetn(&mut devola, 0, 4);
        assert!(buffer.iter().enumerate().all(|(i, n)| range[i] == *n));
    }
}