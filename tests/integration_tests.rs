use nanocore::{assembler::Assembler, nanocore::NanoCore};

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
