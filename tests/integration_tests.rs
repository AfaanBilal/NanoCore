use nanocore::{assembler::Assembler, nanocore::NanoCore};

#[test]
fn test_jmpr() {
    let mut assembler = Assembler::default();
    // LDI R0 0x06  ; Load target address (6) into R0
    // JMPR R0      ; Jump to address in R0
    // HLT          ; Should be skipped
    // LDI R1 0xFF  ; Target instruction
    // HLT
    assembler.assemble(
        "LDI R0 0x06
         JMPR R0
         HLT
         LDI R1 0xFF
         HLT",
    );

    let mut vm = NanoCore::new();
    vm.load_program(&assembler.program, 0);
    vm.run();

    assert_eq!(vm.cpu.registers[1], 0xFF);
}

#[test]
fn test_callr() {
    let mut assembler = Assembler::default();
    // LDI R0 0x06  ; Load target address (6) into R0
    // CALLR R0     ; Call address in R0
    // HLT          ; Should be executed after RET
    // LDI R1 0xFF  ; Target subroutine
    // RET
    assembler.assemble(
        "LDI R0 0x06
         CALLR R0
         HLT
         LDI R1 0xFF
         RET",
    );

    let mut vm = NanoCore::new();
    vm.load_program(&assembler.program, 0);
    vm.run();

    assert_eq!(vm.cpu.registers[1], 0xFF);
}
