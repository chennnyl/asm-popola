In opcode descriptions, [R] means any register (axy), [N] means any integer, [A] means any address. [L] means a label. Addresses are indirect references; e.g. lda #0 will place the value located at memory address 0 in the accumulator. Most [RNA] commands will accept a second [RNA] parameter separated by a comma; this will produce a reference to a memory address where the second parameter acts as an offset.

There are 4 processor flags: Carry, Zero, Parity, and Sign. Various operations set them in different ways.

    ld[R] [RNA]: Load a value in a register
    st[R] [RNA]: Store a register's value
    inc: Increment the accumulator
    dec: Decrement the accumulator
    add [RNA]: Add a value to the accumulator
    sub [RNA]: Subtract a value from the accumulator
    cmp [RNA]: Compare a value to the accumulator
    jmp [L]: Move the code pointer in memory
    jc [L] / jnc [L]: Jump if C is (not) set
    jz [L] / jnz [L]: Jump if Z is (not) set
    jpe [L] / jpo [L]: Jump if P is (not) set
    jm [L] / jp [L]: Jump if S is (not) set
    call [L]: Push the code pointer to the call stack and move to a label
    ret: Pop the topmost entry of the call stack and move the code pointer there For each of these jump instructions, there is a corresponding c (call) instruction that pushes the current code pointer to the call stack and a r (return) instruction that pops it.
    push [R] / pop [R]: Push/pop data onto the stack from a register
