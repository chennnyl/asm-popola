#[derive(Clone, Copy)]
pub enum Flag {
    Carry, Zero, Parity, Sign
}

#[derive(Clone, Copy)]
pub enum Register {
    Accumulator, IndexX, IndexY
}

#[derive(Clone, Copy)]
pub enum AddressingMode {
    /// The byte stored in the corresponding register
    Register(Register),
    /// The byte of the argument
    Immediate(u8),
    /// The byte located at the immediate pointer argument
    Indirect(u16),
    /// The byte located at X:Y
    Index
}

#[derive(Clone, Copy)]
pub enum JumpType {
    Unconditional,
    Flag(Flag, bool)
}

#[derive(Clone)]
pub enum CallType {
    Local(u16),
    Library(String)
}

#[derive(Clone)]
pub enum Instruction {
    /// `ld[Ra] [Rb | N | I | XY]`
    /// - For `Rb`: Sets `Ra` to the value of `Rb`
    /// - For `N`: Sets `Ra` to `N`
    /// - For `I`: Sets `Ra` to the value located at the address `I`
    /// - For `XY`: Sets `Ra` to the value at the address indicated by `XY`
    ///
    /// **Flags affected:** None
    Load(Register, AddressingMode),
    /// `st[Ra] [I | XY]`
    /// - For `I`: Stores the contents of `Ra` to the address `I`
    /// - For `XY`: Stores the contents of `Ra` to the address indicated by `XY`
    ///
    /// **Flags affected:** None
    Store(Register, AddressingMode),
    /// `inc`
    /// - Increments the accumulator
    ///
    /// **Flags affected:**
    /// - `Z` sets if `A` is now `0`, resets otherwise
    /// - `S` if the most significant bit of the accumulator is now `1`, resets otherwise
    /// - `P` if the least significant bit of the accumulator is now `1`, resets otherwise
    Increment,
    /// `dec`
    /// - Decrements the accumulator
    ///
    /// **Flags affected:**
    /// - `Z` sets if `A` is now `0`, resets otherwise
    /// - `S` if the most significant bit of `A` is now `1`, resets otherwise
    /// - `P` if the least significant bit of `A` is now `1`, resets otherwise
    Decrement,
    /// `add [Rb | N | I | XY]`
    /// - For `Rb`: The contents of `Rb` are added to the accumulator
    /// - For `N`: `N` is added to the accumulator
    /// - For `I`: The byte pointed to by `I` is added to the accumulator
    /// - For `XY`: The byte pointed to by `XY` is added to the accumulator
    ///
    /// **Flags affected:**
    /// - `Z` sets if the accumulator is now `0`, resets otherwise
    /// - `S` sets if the most significant bit of the accumulator is now `1`, resets otherwise
    /// - `P` sets if the least significant bit of the accumulator is now `1`, resets otherwise
    /// - `C` sets if the addition results in a carry
    Add(AddressingMode),

    /// `sub [Rb | N | I | XY]`
    ///
    /// Performs twos complement subtraction with the accumulator. See documentation for `Instruction::Add`.
    Subtract(AddressingMode),

    /// `cmp [Rb | N | I | XY]`
    /// - Let the value of the argument be represented by `x`.
    /// - `Z` sets if `A == x`, resets otherwise
    /// - `S` sets if `sgn(A) == sgn(x)`, resets otherwise
    /// - `P` sets if `A % 2 == x % 2`, resets otherwise
    /// - `C` sets if `A < x`, resets if `A >= x`
    Compare(AddressingMode),
    Jump(JumpType, usize),
    Call(CallType),
    Return,
    Push(Register), Pop(Register),
    Nop, _Label(&'static str), _Assert(AddressingMode, u8), _LabeledJump(JumpType, &'static str)
}