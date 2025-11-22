use nanocore::{assembler::Assembler, nanocore::NanoCore};

#[test]
fn test_jmpr() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler.assemble(
        "LDI R0 0x06
         JMPR R0
         HLT
         LDI R1 0xFF
         HLT",
    );

    let mut vm = NanoCore::new();
    vm.load_program(&assembler.program, 0)?;
    vm.run()?;

    assert_eq!(vm.cpu.registers[1], 0xFF);
    Ok(())
}

#[test]
fn test_callr() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler.assemble(
        "LDI R0 0x06
         CALLR R0
         HLT
         LDI R1 0xFF
         RET",
    );

    let mut vm = NanoCore::new();
    vm.load_program(&assembler.program, 0)?;
    vm.run()?;

    assert_eq!(vm.cpu.registers[1], 0xFF);
    Ok(())
}
