use nanocore::{assembler::Assembler, cpu::CPU, nanocore::NanoCore};

fn run(asm: &str) -> Result<NanoCore, Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler.assemble(asm)?;

    let mut nano = NanoCore::new();
    nano.load_program(&assembler.program, 0)?;
    nano.run()?;

    Ok(nano)
}

#[test]
fn test_jmpr() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler
        .assemble(
            "LDI R0 6
         JMPR R0
         HLT
         LDI R1 1
         HLT",
        )
        .unwrap();

    let mut nano = NanoCore::new();
    nano.load_program(&assembler.program, 0)?;
    nano.run()?;

    assert_eq!(nano.cpu.registers[1], 1);
    Ok(())
}

#[test]
fn test_callr() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler
        .assemble(
            "LDI R0 0x06
         CALLR R0
         HLT
         LDI R1 0xFF
         RET",
        )
        .unwrap();

    let mut vm = NanoCore::new();
    vm.load_program(&assembler.program, 0)?;
    vm.run()?;

    assert_eq!(vm.cpu.registers[1], 0xFF);
    Ok(())
}

#[test]
fn test_store_and_push_preserve_flags() -> Result<(), Box<dyn std::error::Error>> {
    // CMP sets Z; writing memory must not disturb it.
    let nano = run("LDI R0 5
         LDI R1 5
         LDI R2 9
         CMP R0 R1
         PUSH R2
         STORE R2 0x40
         STR R2 R3
         HLT")?;

    assert!(nano.cpu.get_flag(CPU::FLAG_Z), "PUSH/STORE cleared Z");
    assert_eq!(nano.cpu.memory[0x40], 9, "STORE did not write");
    Ok(())
}

#[test]
fn test_pop_and_mov_set_flags() -> Result<(), Box<dyn std::error::Error>> {
    // POP loads a register, so it does set Z.
    let nano = run("LDI R0 0
         PUSH R0
         LDI R1 7
         POP R2
         HLT")?;

    assert!(nano.cpu.get_flag(CPU::FLAG_Z), "POP did not set Z");
    assert_eq!(nano.cpu.registers[2], 0);

    // MOV doubles as a test-for-zero.
    let nano = run("LDI R0 0
         LDI R1 7
         MOV R1 R0
         HLT")?;

    assert!(nano.cpu.get_flag(CPU::FLAG_Z), "MOV did not set Z");
    Ok(())
}

#[test]
fn test_assembler_reuse() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();

    assembler.assemble(
        ".CONST VAL 1
         start:
         LDI R0 VAL
         JMP start",
    )?;
    let first = assembler.program.clone();

    // A second run must not append to, or inherit labels/constants from, the first.
    assembler.assemble(
        ".CONST VAL 1
         start:
         LDI R0 VAL
         JMP start",
    )?;

    assert_eq!(assembler.program, first);
    assert_eq!(assembler.labels.len(), 1);
    assert_eq!(assembler.constants.len(), 1);

    // Stale labels from a previous program must not resolve in a later one.
    assert!(assembler.assemble("JMP start").is_err());

    Ok(())
}
