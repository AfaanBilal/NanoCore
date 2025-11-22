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
