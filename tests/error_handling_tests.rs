use nanocore::{EmulatorError, assembler::Assembler, nanocore::NanoCore};

#[test]
fn test_division_by_zero_div() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler
        .assemble(
            "LDI R0 10
         LDI R1 0
         DIV R0 R1
         HLT",
        )
        .unwrap();

    let mut nano = NanoCore::new();
    nano.load_program(&assembler.program, 0)?;

    // Execute LDI R0 10
    nano.cycle()?;
    // Execute LDI R1 0
    nano.cycle()?;

    // Execute DIV R0 R1 - should return error
    let result = nano.cycle();
    assert!(result.is_err());

    match result {
        Err(EmulatorError::DivisionByZero { op }) => {
            assert!(op.contains("DIV"));
        }
        _ => panic!("Expected DivisionByZero error"),
    }

    Ok(())
}

#[test]
fn test_division_by_zero_mod() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler
        .assemble(
            "LDI R0 10
         LDI R1 0
         MOD R0 R1
         HLT",
        )
        .unwrap();

    let mut nano = NanoCore::new();
    nano.load_program(&assembler.program, 0)?;

    nano.cycle()?; // LDI R0 10
    nano.cycle()?; // LDI R1 0

    let result = nano.cycle(); // MOD R0 R1
    assert!(result.is_err());

    if let Err(EmulatorError::DivisionByZero { op }) = result {
        assert!(op.contains("MOD"));
    } else {
        panic!("Expected DivisionByZero error");
    }

    Ok(())
}

#[test]
fn test_division_by_zero_divi() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler
        .assemble(
            "LDI R0 42
         DIVI R0 0
         HLT",
        )
        .unwrap();

    let mut nano = NanoCore::new();
    nano.load_program(&assembler.program, 0)?;

    nano.cycle()?; // LDI R0 42

    let result = nano.cycle(); // DIVI R0 0
    assert!(matches!(result, Err(EmulatorError::DivisionByZero { .. })));

    Ok(())
}

#[test]
fn test_division_by_zero_modi() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler
        .assemble(
            "LDI R0 42
         MODI R0 0
         HLT",
        )
        .unwrap();

    let mut nano = NanoCore::new();
    nano.load_program(&assembler.program, 0)?;

    nano.cycle()?;

    let result = nano.cycle();
    assert!(matches!(result, Err(EmulatorError::DivisionByZero { .. })));

    Ok(())
}

#[test]
fn test_stack_overflow() -> Result<(), Box<dyn std::error::Error>> {
    let mut nano = NanoCore::new();

    // Stack pointer decrements on PUSH, STACK_MIN is 0xEA (234)
    // PUSH opcode is 0x07
    let mut program = vec![];

    for _ in 0..10 {
        program.push(0x07); // PUSH opcode
        program.push(0x00); // R0
    }
    program.push(0x00); // HLT

    nano.load_program(&program, 0)?;

    // Set SP close to STACK_MIN (0xEA) to trigger overflow
    nano.cpu.sp = 0xEB; // One above minimum

    let mut error_occurred = false;
    for _ in 0..11 {
        match nano.cycle() {
            Ok(_) => continue,
            Err(EmulatorError::StackOverflow { sp }) => {
                error_occurred = true;
                assert_eq!(sp, 0xEA); // Should be at STACK_MIN
                break;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    assert!(error_occurred, "Expected stack overflow error");
    Ok(())
}

#[test]
fn test_stack_underflow() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    // Try to RET without a corresponding CALL
    assembler.assemble("RET").unwrap();

    let mut nano = NanoCore::new();
    nano.load_program(&assembler.program, 0)?;

    let result = nano.cycle();
    assert!(matches!(result, Err(EmulatorError::StackUnderflow { .. })));

    Ok(())
}

#[test]
fn test_program_too_large() {
    let mut nano = NanoCore::new();
    let large_program = vec![0u8; 300]; // 300 bytes - exceeds 256 limit

    let result = nano.load_program(&large_program, 0x00);

    assert!(result.is_err());
    match result {
        Err(EmulatorError::ProgramTooLarge { size, start, max }) => {
            assert_eq!(size, 300);
            assert_eq!(start, 0);
            assert_eq!(max, 256);
        }
        _ => panic!("Expected ProgramTooLarge error"),
    }
}

#[test]
fn test_program_too_large_with_offset() {
    let mut nano = NanoCore::new();
    let program = vec![0u8; 200]; // 200 bytes

    // At offset 0x80 (128), this would go to 328, exceeding 256
    let result = nano.load_program(&program, 0x80);

    assert!(matches!(result, Err(EmulatorError::ProgramTooLarge { .. })));
}

#[test]
fn test_normal_division_works() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler
        .assemble(
            "LDI R0 10
         LDI R1 2
         DIV R0 R1
         HLT",
        )
        .unwrap();

    let mut nano = NanoCore::new();
    nano.load_program(&assembler.program, 0)?;
    nano.run()?;

    assert_eq!(nano.cpu.registers[0], 5); // 10 / 2 = 5
    Ok(())
}

#[test]
fn test_normal_stack_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = Assembler::default();
    assembler
        .assemble(
            "LDI R0 42
         PUSH R0
         LDI R0 0
         POP R1
         HLT",
        )
        .unwrap();

    let mut nano = NanoCore::new();
    nano.load_program(&assembler.program, 0)?;
    nano.run()?;

    assert_eq!(nano.cpu.registers[1], 42);
    Ok(())
}

#[test]
fn test_invalid_opcode() -> Result<(), Box<dyn std::error::Error>> {
    let mut nano = NanoCore::new();
    // LDI R0 1; <unmapped byte>
    nano.load_program(&[0x02, 0x00, 0x01, 0xFF], 0)?;

    match nano.run() {
        Err(EmulatorError::InvalidOpcode { opcode, pc }) => {
            assert_eq!(opcode, 0xFF);
            assert_eq!(pc, 0x03);
        }
        other => panic!("Expected InvalidOpcode, got {other:?}"),
    }

    Ok(())
}

#[test]
fn test_running_into_data_is_an_error() -> Result<(), Box<dyn std::error::Error>> {
    // Falling off the end of the code into a .DB block must fault, not NOP-slide.
    let mut assembler = Assembler::default();
    assembler.assemble(
        "LDI R0 1
         .DB 0xFF 0xFE",
    )?;

    let mut nano = NanoCore::new();
    nano.load_program(&assembler.program, 0)?;

    assert!(matches!(
        nano.run(),
        Err(EmulatorError::InvalidOpcode { opcode: 0xFF, .. })
    ));

    Ok(())
}

#[test]
fn test_opcode_decoding_boundaries() {
    use nanocore::Op;

    // Every mapped opcode decodes and round-trips back to its byte.
    for byte in 0x00..=0x27u8 {
        let op = Op::decode(byte).unwrap_or_else(|| panic!("{byte:#04X} should decode"));
        assert_eq!(u8::from(op), byte);
    }

    // Everything above is unmapped.
    for byte in 0x28..=0xFFu8 {
        assert!(Op::decode(byte).is_none(), "{byte:#04X} should not decode");

        // ...but the display-only From impl still yields NOP, which the TUI
        // memory view relies on to walk arbitrary bytes.
        assert_eq!(Op::from(byte), Op::NOP);
    }
}

#[test]
fn test_out_of_range_register_operand_does_not_panic() -> Result<(), Box<dyn std::error::Error>> {
    let mut nano = NanoCore::new();
    nano.load_program(
        &[
            0x02, 0xF3, 0x2A, // LDI R0xF3 42 -> R3
            0x07, 0xFF, // PUSH R0xFF -> R15
            0x00, // HLT
        ],
        0x00,
    )?;
    nano.run()?;

    assert_eq!(nano.cpu.registers[3], 42);
    assert_eq!(nano.cpu.sp, 0xFE);
    Ok(())
}
