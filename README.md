# NanoCore: An 8-bit CPU Emulator

[](https://opensource.org/licenses/MIT)

## 🌟 Introduction

`NanoCore` is a meticulously crafted emulator for a custom 8-bit CPU.

Designed with extreme minimalism in mind, this CPU operates within a strict 256-byte memory space, with all registers, the Program Counter (PC), and the Stack Pointer (SP) being 8-bit.

This project serves as an educational exercise in understanding the fundamental principles of computer architecture, low-level instruction set design, memory management under severe constraints, and assembly language programming.

## ✨ Key Features

  * **True 8-bit Architecture:** All general-purpose registers (R0-R15), Program Counter (PC), and Stack Pointer (SP) are 8-bit.
  * **256-byte Memory:** The entire addressable memory space is limited to 256 bytes (`0x00` to `0xFF`), making it a challenge to write highly optimized and compact code.
  * **Variable-Length Instruction Set:** Supports both 1-byte and 2-byte instructions to maximize opcode efficiency and flexibility within the limited address space.
  * **Modular Design:** CPU cycle is broken down into distinct Fetch, Decode, and Execute phases for clarity.
  * **Inbuilt Assembler:** The NanoCore Assembler makes it easier to program it by writing NanoCore Assembly instead of direct machine code.

## 🧮 Instruction Set Architecture (ISA)

NanoCore features a small but functional instruction set designed for its 8-bit constraints.

### Instruction Format

  * **1-byte instructions:** One 8-bit opcode. Some opcodes have register operands in their lower bits.
  * **2-byte instructions:** An 8-bit opcode followed by an 8-bit operand (e.g., an immediate value or an address).

### Implemented Instructions

| Opcode | Mnemonic        | Description                                                             | Encoding (Example)                                     |
| :----- | :-------------- | :---------------------------------------------------------------------- | :----------------------------------------------------- |
| `0x00` | `HLT`           | Halts CPU execution.                                                    | `0x00`                                                 |
| `0x1X` | `LDI REG, #val` | Loads an 8-bit immediate value `val` into register `REG`.               | `0x10 \| REG`, `Imm8` (2 bytes)                        |
| `0x2X` | `INC REG`       | Increment `REG` by `1` (wrapping addition).                             | `0x20 \| REG`                                          |
| `0x30` | `ADD Rd Rs`     | Adds the value of register `Rs` to register `Rd`. Flags updated.        | `0x30, (Rd << 4) \| Rs` (2 bytes)                      |
| `0x31` | `SUB Rd Rs`     | Subtracts the value of register `Rs` from register `Rd`. Flags updated. | `0x31, (Rd << 4) \| Rs` (2 bytes)                      |
| `0x40` | `JMP Addr`      | Unconditionally jumps to the 8-bit address `Addr`.                      | `0x40`, `Addr8` (2 bytes)                              |
| `0x41` | `JZ Addr`       | Jumps to the 8-bit address `Addr` if the Zero Flag (Z) is set.          | `0x41`, `Addr8` (2 bytes)                              |
| `0x42` | `JNZ Addr`      | Jumps to the 8-bit address `Addr` if the Zero Flag (Z) is not set.      | `0x42`, `Addr8` (2 bytes)                              |
| `0x5X` | `PRINT REG`     | Outputs the ASCII character stored in `REG` to console.                 | `0x50 \| REG` (e.g., `0x10` for `R0`, `0x11` for `R1`) |
| `0x6X` | `SHL REG`       | Shifts the bits in `REG` left by 1 (`<< 1`).                            | `0x60 \| REG` (e.g., `0x10` for `R0`, `0x11` for `R1`) |
| `0x7X` | `SHR REG`       | Shifts the bits in `REG` right by 1 (`>> 1`).                           | `0x70 \| REG` (e.g., `0x10` for `R0`, `0x11` for `R1`) |

## 🚀 Getting Started

To run the NanoCore emulator, you'll need to setup Rust locally.

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/AfaanBilal/nanocore.git
    cd nanocore
    ```
2.  **Run the example program:**
    The `programs/test.ncb` file contains a small, assembled program that demonstrates the CPU's basic functionality.
    ```bash
    cargo run -- programs/test.ncb
    ```
    You should see the emulator's debug output and the program's output to your console. The source assembly file is `programs/test.nca`.

## 🛠️ Assembling

To assemble a program (say `example.nca`), run the NanoCore Assembler (`nca`):
```bash
cargo r --bin nca -- -i example.nca -o example.ncb
```
This should assemble the `example.nca` (NanoCore Assembly) to `example.ncb` (NanoCore Binary).

## ⚙️ Running

To run this assembled binary, run:
```bash
cargo r -- example.ncb
```

## 📂 Code Structure

  * `CPU (cpu.rs)`: Defines the CPU's internal state, including registers, program counter, stack pointer, memory, and flag bits.
  * `NanoCore (nanocore.rs)`: The main emulator struct, responsible for loading programs, running cycles, and managing the `CPU`.
      * `NanoCore::new()`: Initializes a fresh computer state.
      * `NanoCore::load_program()`: Places machine code into the simulated memory.
      * `NanoCore::run()`: Executes the CPU cycle loop until halted.
      * `NanoCore::cycle()`: Performs a single CPU cycle (Fetch, Decode, Execute).
      * `NanoCore::fetch_decode()`: Reads the instruction byte(s) from memory and determines its type and operands.
      * `NanoCore::execute_instruction()`: Performs the operation defined by the decoded instruction, updating the CPU state.
  * `Assembler (assembler.rs)`: The NanoCore assembler.

---

## 🤝 Contributing

All contributions are welcome. Please create an issue first for any feature request
or bug. Then fork the repository, create a branch and make any changes to fix the bug
or add the feature and create a pull request. That's it!
Thanks!

---

## 📄 License

**NanoCore** is released under the MIT License.
Check out the full license [here](LICENSE).
