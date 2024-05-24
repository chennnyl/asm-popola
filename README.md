# Devola & Popola
This project is a Rust-based implementation of an assembly language very loosely based on the instruction set used for the MOS 6502, as well as a corresponding virtual machine/runtime (collectively, **Popola**). It is currently split into two crates, `devola` (the assembler and virtual machine) and `popola` (a to-be-written IDE-type application in SDL2). It takes heavy inspiration from the PICO-8 project in creating a more accessible way to write for 8-bit fantasy consoles. The names come from [Devola](https://nier.fandom.com/wiki/Devola) and [Popola](https://nier.fandom.com/wiki/Popola), twin android sisters in the *Nier* video game franchise.

`popola` takes the role of a kind of PPU -- it is responsible for managing and interpreting VRAM, input, etc., while `devola` handles the underlying code execution. 

- Offsetting the index `XY` quasi-register using `+N` notation

## Popola System Specifications
- **Memory**: 64KiB (16-bit addresses), split into ~60KiB of user memory and 4KiB of VRAM
- **Registers**: 1 accumulator (`A`), 2 general-purpose (`B` and `C`), 2 index (`X` and `Y` -- `X` is the high byte and `Y` is the low byte of an address)
- **Flags**: `C`arry, `P`arity, `Z`ero, `S`ign
- **Memory-mapped I/O (MMIO)**: 16 bytes

Code and memory are currently separated -- thus, it is not currently possible to write self-modifying code. This may change in the future.

## Instruction Set
Popola assembly is case-insensitive.
### Numbers
Numeric arguments to instructions are usually a single byte, except for when providing a 16-bit address for indirect operations. They can be specified as follows:
- Decimal: no suffix; input as a regular number
- Binary: `b` suffix
- Hexadecimal: `h` suffix
- Indirect (address): `#` prefix; supports any of the three bases
### Flags
The four Popola flags can be set by the various arithmetic instructions, as well as by `CMP`.
- `C` is set if an operation results in a carry (overflow) and unset otherwise.
- `P` is set if the result of an operation is odd (that is, if the least significant bit is set) and unset otherwise.
- `S` is set if the result of an operation is negative when interpreted as a signed integer (that is, if the most significant bit is set) and unset otherwise.
- `Z` is set if the result of an operation is `0` and unset otherwise.
### Addressing modes
The following notation is used in describing instruction arguments: 
- **Ra**: A target register; any of `A`, `B`, `C`, `X`, `Y`
- **Rb**: A source register; any of `A`, `B`, `C`, `X`, `Y`
- **N**: An immediate byte value
- **I**: An 16-bit address (indirect access) -- the instruction is provided the byte located at the corresponding address in memory
- **XY**: The address specified by the `XY` index register -- the instruction is provided the byte located at the corresponding address in memory
- **F**: A flag; any of `C`, `P`, `Z`, `S`
- **label**: A labeled location in code

Text in `()` is required, while text in `[]` is optional. The possible values for instruction arguments are separated by `|` characters.

### `LD(Ra) (Rb | N | I | XY)`: Load into a register
### `ST(Rb) (I | XY)`: Store a register into memory
### `INC`/`DEC`: Increment/decrement the accumulator
`Z` is set if the accumulator over/underflows to `0`. The other flags are set accordingly.
### `ADD (Rb | N | I | XY)`/`SUB (Rb | N | I | XY)`: Add to/subtract from the accumulator
`Z` is set if the accumulator is now `0`. The other flags are set accordingly.
### `CMP (Rb | N | I | XY)`: Compare a value to the accumulator
Let `n` represent the argument to `cmp` and `A` the value of the accumulator.
- `C` is set if `A < n` and unset if `A >= n`.
- `P` is set if `A % 2 == n % 2` (`A` and `n` have the same parity) and unset otherwise.
- `S` is set if `sgn(A) == sgn(n)` (`A` and `n` have the same sign) and unset otherwise.
- `Z` is set if `A == n` and unset otherwise.
### (TO BE ADDED) `ADXY (Rb | N | I | XY)`/`SBXY (Rb | N | I | XY)`: Perform 16-bit addition/subtraction
### `JMP (label)`: Unconditionally jump to a location in code
### `J[N](F) (label)`: Conditionally jump to a location in code
If `N` is not present, jumps to the given label if the given flag is set; otherwise, only jumps if the given flag is unset. For example, `JNZ main` jumps to the label `main` only if `Z` is not set.
### `CALL (label)`: Call a subroutine
Pushes the current program counter to the stack and jumps to the given label.
### `RET`: Return from a subroutine
Pops the program counter from the stack and jumps back to the popped value.
### `PUSH (Rb)`: Push to the stack
The stack pointer is decremented and the contents of `Rb` are placed at the new stack pointer. (The stack grows down.)
### `POP (Ra)`: Pop from the stack
The byte located at the stack pointer is placed into `Ra` and the stack pointer is incremented. (The stack shrinks up.)
### `NOP`: No-op
Does nothing. Substitutes labels in compiled code.

## "Hardware" information
### MMIO
The 16-byte range `0x0FF0`-`0x0FFF` in memory is currently reserved for memory mapped I/O. They are currently mapped as follows:
- `MMIO+0x0`: Most significant byte of the stack pointer
- `MMIO+0x1`: Least significant byte of the stack pointer
- `MMIO+0x2-0xF`: Unassigned
### Subroutine convention
Convention for unary functions that return a single byte is to place both arguments and return values in the `B` register. For more complex functions, you can either use multiple registers or utilize a stack frame.

## Example programs
More examples are available at `devola/sample`.
### Square an integer
```
	lda  0      ; i = 0
	ldb 5       ; n = 5
	ldc 0       ; square = 0

loop:               ; while true
	cmp b       ; if i == n break
	jz end_loop
	push a      ; square += n
	lda c
	add b
	ldc a
	pop a
	inc         ; i++
	jmp loop
end_loop:           ; c contains 5^2
```

### Integer square subroutine
```
    jmp main
; place number to square in b, square will be returned there
square:
    push a
    push c
    lda 0       ; i = 0
    ldc 0       ; square = 0
loop:           ; while true
    cmp b       ; if i == n break
    jz end_loop
    push a      ; square += n
    lda c
    add b
    ldc a
    pop a
    inc         ; i++
    jmp loop
end_loop:
    ldb c
    pop c
    pop a
    ret
main:
    ldb 13
    call square     ; b = 169
    ldb 12
    call square     ; b = 144
    ldb 3
    call square     ; b = 9
```

### Simple stack frame
We could translate the program (example in Python)
```python
def add_doubles(n1, n2):
    return 2*n1 + 2*n2

def main():
    add_doubles(10, 5)
```

```
    jmp main
add_doubles:
    ;; set stack frame
    push x      ;; save old index
    push y      ; stack-6
    ldx #0FF0h  ;; get current stack pointer
    ldy #0FF1h
    sbxy 2      ; two 1-byte local variables 
    stx #0FF0h  ;; update stack pointer
    sty #0FF1h
    ;; done setting stack frame
    
    lda XY+7    ; n1
    add a       ; n1+n1
    sta XY+1    ; store in first local variable
    lda XY+8    ; n2
    add a       ; n2+n2
    sta XY+2    ; store in second local variable
    
    ldb XY+1    ; access first local variable
    lda XY+2    ; access second local variable
    add b       ; 2*n1 + 2*n2
    sta b       ; place in return
    
    ;; reset stack frame
    adxy 2      ; throw away local variables
    stx #0FF0h  ;; restore stack pointer
    sty #0FF1h 
    pop y       ;; restore old index
    pop x
    ;; done resetting stack frame
    
    ret
main:
    push 5
    push 10
    call add_doubles ; b has the result
```