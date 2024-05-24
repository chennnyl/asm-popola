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