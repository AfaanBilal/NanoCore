use nanocore::nanocore::NanoCore;

#[test]
fn test_basic_execution() {
    let mut nano = NanoCore::new();
    let program = vec![
        0x02, 0x00, 0x0A, // LDI R0, 10
        0x0D, 0x00, // INC R0
        0x00, // HLT
    ];

    nano.load_program(&program, 0).unwrap();
    nano.run().unwrap();

    assert_eq!(nano.cpu.registers[0], 11);
}

#[test]
fn test_division_by_zero() {
    let mut nano = NanoCore::new();
    let program = vec![
        0x02, 0x00, 0x0A, // LDI R0, 10
        0x02, 0x01, 0x00, // LDI R1, 0
        0x1C, 0x01, // DIV R0 R1
    ];

    nano.load_program(&program, 0).unwrap();

    while !nano.cpu.is_halted {
        if let Err(e) = nano.cycle() {
            assert!(matches!(e, nanocore::EmulatorError::DivisionByZero { .. }));
            return;
        }
    }

    panic!("Expected division by zero error");
}
