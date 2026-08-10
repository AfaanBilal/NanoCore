# NanoCore: An 8-bit CPU Emulator

`NanoCore` is a meticulously crafted emulator for a custom 8-bit CPU. Designed with extreme minimalism in mind, this CPU operates within a strict 256-byte memory space, with all registers, the Program Counter (PC), and the Stack Pointer (SP) being 8-bit.

This project serves as an educational exercise in understanding the fundamental principles of computer architecture, low-level instruction set design, memory management under severe constraints, and assembly language programming.

**Website:** [nanocore.afaan.dev](https://nanocore.afaan.dev) &nbsp;|&nbsp; **DeepWiki:** [AfaanBilal/NanoCore](https://deepwiki.com/AfaanBilal/NanoCore)

<img src="assets/NanoCoreTUI.gif" alt="NanoCore TUI">

---

## ✨ Key Features

- **True 8-bit Architecture:** All general-purpose registers (R0–R15), Program Counter (PC), and Stack Pointer (SP) are 8-bit.
- **256-byte Memory:** The entire addressable memory space is limited to 256 bytes (`0x00` to `0xFF`).
- **Variable-Length Instruction Set:** 1-byte, 2-byte, and 3-byte instructions to maximize opcode efficiency within the limited address space.
- **Modular Design:** CPU cycle broken down into distinct Fetch, Decode, and Execute phases.
- **Inbuilt Two-Pass Assembler:** Write NanoCore Assembly (`.nca`) instead of raw machine code.
- **Terminal User Interface:** Fully functional TUI with breakpoints for interactive debugging.
- **Typed Error Handling:** Stack overflow/underflow, division by zero, invalid operands, and invalid opcodes all surface as structured Rust errors.

---

## 📦 Installation

### Option 1 — Compiled Binaries

Download pre-built binaries for your platform from the [GitHub Releases](https://github.com/AfaanBilal/NanoCore/releases) page.

### Option 2 — cargo install

```bash
cargo install nanocore
```

### Option 3 — Build from Source

Requires Rust (stable).

```bash
git clone https://github.com/AfaanBilal/NanoCore.git
cd NanoCore
cargo build --release
```

---

## 🚀 Usage

### Run a program

```bash
# Run a pre-assembled binary
cargo run -- programs/test.ncb

# Auto-assemble and run a .nca source file
cargo run -- programs/fib.nca

# Print CPU state after each cycle
cargo run -- programs/test.nca -s

# Print each instruction as it executes
cargo run -- programs/test.nca -i
```

### Assemble to binary

```bash
cargo run --bin nca -- -i example.nca -o example.ncb
```

### Launch the TUI debugger

```bash
cargo run --bin tui -- programs/counter.nca
```

### Run the test suite

```bash
cargo test
```

---

## 🧮 Architecture

| Component | Details |
| :--- | :--- |
| Registers | R0–R15, all 8-bit |
| Program Counter | 8-bit (0x00–0xFF) |
| Stack Pointer | 8-bit (stack: 0xEB–0xFF; 0xEA is the overflow guard) |
| Flags | Zero (Z), Carry (C), Negative (N) |
| Memory | 256 bytes total |
| Stack size | 21 bytes (0xEB–0xFF) |
| Max cycles | 1024 per run |

### Memory Map

All 256 bytes are one flat, unprotected address space. The layout below is convention — nothing enforces it.

| Range           |     Size | Purpose                                                  |
| :-------------- | -------: | :------------------------------------------------------- |
| `0x00`–`0xA9`   | 170 bytes | Program and data — programs load at `0x00` by default    |
| `0xAA`–`0xE9`   |  64 bytes | Screen — the TUI debugger renders this as an 8×8 grid    |
| `0xEA`          |   1 byte  | Stack overflow guard — never written                     |
| `0xEB`–`0xFF`   |  21 bytes | Stack                                                     |

### Stack Model

The stack **grows downward** from the top of memory. `SP` starts at `0xFF` and always points at the **next free slot**, so the value on top is at `memory[SP + 1]` and an empty stack is `SP == 0xFF`.

| Instruction     | Effect                                                                  |
| :-------------- | :---------------------------------------------------------------------- |
| `PUSH Rd`       | `memory[SP] = Rd`, then `SP -= 1`                                       |
| `POP Rd`        | `SP += 1`, then `Rd = memory[SP]`                                       |
| `CALL`/`CALLR`  | Pushes the return address (`PC + 2`), then jumps                        |
| `RET`           | Pops into `PC`                                                          |

```text
SP = 0xFF          ; empty
PUSH R0            ; memory[0xFF] = R0,  SP = 0xFE
PUSH R1            ; memory[0xFE] = R1,  SP = 0xFD
POP  R2            ; SP = 0xFE,  R2 = memory[0xFE]  (the old R1)
```

`PUSH` on a full stack (`SP == 0xEA`) raises `StackOverflow`; `POP` on an empty one (`SP == 0xFF`) raises `StackUnderflow`. Both are checked *before* the access, so `0xEA` is never written — hence the guard byte.

> [!IMPORTANT]
> Return addresses and pushed data share the same 21 bytes. Nesting `CALL`s 21 deep exhausts the stack on its own, and an unbalanced `PUSH` inside a subroutine leaves `RET` popping a data byte and jumping to it.

> [!WARNING]
> Nothing protects the stack region. `load_program` only checks that the program fits in 256 bytes, so a program longer than 235 bytes overwrites the stack (and one over 170 bytes overwrites the screen). `STORE`/`STR` into `0xEB`–`0xFF` corrupts it silently — there is no fault.

---

## 🧮 Instruction Set Architecture (ISA)

> [!TIP]
> See `programs/` for example programs and compiled binaries.

NanoCore features a small but complete instruction set across 7 categories.

### Instruction Format

- **1-byte:** Opcode only.
- **2-byte:** Opcode + one 8-bit operand (register or address).
- **3-byte:** Opcode + two 8-bit operands (register + immediate or address).

### Implemented Instructions

| Opcode | Bytes | Mnemonic         | Description                             |
| :----- | ----: | :--------------- | :-------------------------------------- |
| `0x00` |     1 | `HLT`            | Halt execution                          |
| `0x01` |     1 | `NOP`            | No operation                            |
| `0x02` |     3 | `LDI Rd val`     | Load immediate `val` into `Rd`          |
| `0x03` |     3 | `LDA Rd addr`    | Load from memory address into `Rd`      |
| `0x04` |     2 | `LDR Rd Rs`      | Load from address in `Rs` into `Rd`     |
| `0x05` |     2 | `MOV Rd Rs`      | Copy `Rs` into `Rd`                     |
| `0x06` |     3 | `STORE addr Rd`  | Store `Rd` into memory address          |
| `0x07` |     2 | `PUSH Rd`        | Push `Rd` onto stack                    |
| `0x08` |     2 | `POP Rd`         | Pop top of stack into `Rd`              |
| `0x09` |     2 | `ADD Rd Rs`      | `Rd = Rd + Rs`                          |
| `0x0A` |     3 | `ADDI Rd val`    | `Rd = Rd + val`                         |
| `0x0B` |     2 | `SUB Rd Rs`      | `Rd = Rd - Rs`                          |
| `0x0C` |     3 | `SUBI Rd val`    | `Rd = Rd - val`                         |
| `0x0D` |     2 | `INC Rd`         | `Rd = Rd + 1`                           |
| `0x0E` |     2 | `DEC Rd`         | `Rd = Rd - 1`                           |
| `0x0F` |     2 | `AND Rd Rs`      | `Rd = Rd & Rs`                          |
| `0x10` |     2 | `OR Rd Rs`       | `Rd = Rd \| Rs`                         |
| `0x11` |     2 | `XOR Rd Rs`      | `Rd = Rd ^ Rs`                          |
| `0x12` |     2 | `NOT Rd`         | `Rd = ~Rd`                              |
| `0x13` |     2 | `CMP Rd Rs`      | Set Z/N/C from `Rd - Rs` (no store; C=borrow if `Rd < Rs`) |
| `0x14` |     2 | `SHL Rd`         | `Rd = Rd << 1` (C = old bit 7)          |
| `0x15` |     2 | `SHR Rd`         | `Rd = Rd >> 1` (C = old bit 0)          |
| `0x16` |     2 | `JMP addr`       | Unconditional jump                      |
| `0x17` |     2 | `JZ addr`        | Jump if Zero flag set                   |
| `0x18` |     2 | `JNZ addr`       | Jump if Zero flag clear                 |
| `0x19` |     2 | `PRINT Rd`       | Print `Rd` as ASCII character           |
| `0x1A` |     2 | `MUL Rd Rs`      | `Rd = Rd * Rs`                          |
| `0x1B` |     3 | `MULI Rd val`    | `Rd = Rd * val`                         |
| `0x1C` |     2 | `DIV Rd Rs`      | `Rd = Rd / Rs`                          |
| `0x1D` |     3 | `DIVI Rd val`    | `Rd = Rd / val`                         |
| `0x1E` |     2 | `MOD Rd Rs`      | `Rd = Rd mod Rs`                        |
| `0x1F` |     3 | `MODI Rd val`    | `Rd = Rd mod val`                       |
| `0x20` |     2 | `CALL addr`      | Call subroutine (push return address)   |
| `0x21` |     1 | `RET`            | Return from subroutine                  |
| `0x22` |     2 | `ROL Rd`         | Rotate `Rd` left by 1 (C = old bit 7)   |
| `0x23` |     2 | `ROR Rd`         | Rotate `Rd` right by 1 (C = old bit 0)  |
| `0x24` |     2 | `IN Rd`          | Read byte from stdin into `Rd`          |
| `0x25` |     2 | `JMPR Rd`        | Jump to address in `Rd`                 |
| `0x26` |     2 | `CALLR Rd`       | Call subroutine at address in `Rd`      |
| `0x27` |     2 | `STR Rd Rs`      | Store `Rd` to address held in `Rs`      |

> [!TIP]
> All arithmetic is wrapping. `R0 = 0x00`, `R1 = 0x01`, ..., `R15 = 0x0F`.

Opcodes `0x28`–`0xFF` are unassigned. Executing one raises `EmulatorError::InvalidOpcode`, reporting the byte and the PC — so running off the end of your code into a `.DB` block or jumping to a bad address fails loudly instead of drifting through it.

### Flag Effects

The rule: **an instruction that writes a register sets Z and N; one that writes memory or the stack sets nothing.** `CMP` is the only exception — it sets flags without writing a register, which is its entire job.

**Z** is set when the result is `0x00`, **N** from bit 7 of the result. **C** is written only where marked — every other instruction leaves the previous **C** intact.

| Instruction                            |  Z  |  C  |  N  | Notes                                         |
| :------------------------------------- | :-: | :-: | :-: | :-------------------------------------------- |
| `LDI`, `LDA`, `LDR`, `MOV`, `IN`, `POP` |  ✔  |  —  |  ✔  | From the loaded / popped value                |
| `STORE`, `STR`, `PUSH`                 |  —  |  —  |  —  | Write memory — no register result to flag     |
| `INC`, `DEC`                      |  ✔  |  —  |  ✔  | Wraps silently — no C on `0xFF` → `0x00`           |
| `ADD`, `ADDI`, `SUB`, `SUBI`      |  ✔  |  ✔  |  ✔  | C = carry-out (`ADD`) / borrow (`SUB`)             |
| `MUL`, `MULI`                     |  ✔  |  ✔  |  ✔  | C set when the product exceeds `0xFF`              |
| `DIV`, `DIVI`, `MOD`, `MODI`      |  ✔  |  ✔  |  ✔  | C always cleared — 8-bit division cannot overflow  |
| `AND`, `OR`, `XOR`, `NOT`         |  ✔  |  —  |  ✔  |                                                    |
| `CMP`                             |  ✔  |  ✔  |  ✔  | From `Rd - Rs`; C = borrow. No register written    |
| `SHL`, `ROL`                      |  ✔  |  ✔  |  ✔  | C = old bit 7                                      |
| `SHR`, `ROR`                      |  ✔  |  ✔  |  ✔  | C = old bit 0                                      |
| `JMP`, `JMPR`, `JZ`, `JNZ`        |  —  |  —  |  —  | `JZ` / `JNZ` read Z, never write it                |
| `CALL`, `CALLR`, `RET`            |  —  |  —  |  —  |                                                    |
| `PRINT`                           |  —  |  —  |  —  |                                                    |
| `HLT`, `NOP`                      |  —  |  —  |  —  |                                                    |

> [!NOTE]
> Loads set Z/N, so a `MOV` or `LDI` between a `CMP` and a `JZ` clobbers the comparison. The flip side is useful: since there is no `TST`, `MOV Rd Rs` doubles as a test-for-zero on `Rs`.

> [!TIP]
> `PUSH` and `STORE` preserve flags, so a subroutine prologue can save registers without destroying the caller's comparison. `POP` does not — it loads a register.

---

## 🛠️ Assembly Language

NanoCore Assembly (`.nca`) files are plain text. The assembler performs two passes — first to map labels and constants, then to emit bytecode.

### Syntax

```assembly
; Comment
.CONST MAX 10          ; Named constant
.DB 0x01 0x02 0x03     ; Embed raw bytes
.STRING "Hello"        ; Embed ASCII string (null-terminated)

start:                 ; Label
    LDI R0 0
    LDI R1 MAX         ; Use constant
loop:
    ADD R0 R1
    DEC R2
    JNZ loop
    HLT
```

### Directives

| Directive | Description |
| :--- | :--- |
| `.CONST name val` | Define a named constant |
| `.DB byte ...` | Embed raw bytes at current position |
| `.STRING "text"` | Embed a null-terminated ASCII string |

---

## 📂 Code Structure

| File | Description |
| :--- | :--- |
| `src/cpu.rs` | CPU state — registers, PC, SP, memory, flags |
| `src/nanocore.rs` | Main emulator — load, run, cycle, fetch/decode/execute |
| `src/assembler.rs` | Two-pass assembler core |
| `src/lib.rs` | Library exports and `Op` enum (instruction set) |
| `src/error.rs` | Typed error definitions |
| `src/bin/nca.rs` | `nca` assembler binary |
| `src/bin/tui.rs` | `tui` debugger binary entry point |
| `src/tui/` | TUI implementation (ratatui) |
| `programs/` | Example `.nca` source files and `.ncb` binaries |

---

## 📝 Example Program — Fibonacci Sequence

```assembly
; Print the fibonacci sequence (two-digit)
start:
    LDI R0 0
    LDI R1 1
    LDI R2 12
    LDI R12 32
loop:
    JMP print_digits

post_print:
    MOV R3 R1
    ADD R1 R0
    MOV R0 R3
    DEC R2
    JNZ loop
end:
    HLT

print_digits:
    PUSH R10
    PUSH R11
    MOV R10 R0
    DIVI R10 10
    JZ unit_digit
    ADDI R10 48
    PRINT R10
unit_digit:
    MOV R11 R0
    MODI R11 10
    ADDI R11 48
    PRINT R11
print_space:
    PRINT R12
    POP R11
    POP R10
    JMP post_print
```

---

## 🤝 Contributing

All contributions are welcome. Please create an issue first for any feature request or bug. Then fork the repository, create a branch, make your changes, and open a pull request.

---

## 📄 License

**NanoCore** is released under the MIT License. See [LICENSE](LICENSE) for details.
