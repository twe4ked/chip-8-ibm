const MEMORY_LENGTH: usize = 0xfff;
const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const PROGRAM_OFFSET: u16 = 0x200;
const IBM_LOGO_ROM: [u8; 132] = [
    0x00, 0xe0, 0xa2, 0x2a, 0x60, 0x0c, 0x61, 0x08, 0xd0, 0x1f, 0x70, 0x09, 0xa2, 0x39, 0xd0, 0x1f,
    0xa2, 0x48, 0x70, 0x08, 0xd0, 0x1f, 0x70, 0x04, 0xa2, 0x57, 0xd0, 0x1f, 0x70, 0x08, 0xa2, 0x66,
    0xd0, 0x1f, 0x70, 0x08, 0xa2, 0x75, 0xd0, 0x1f, 0x12, 0x28, 0xff, 0x00, 0xff, 0x00, 0x3c, 0x00,
    0x3c, 0x00, 0x3c, 0x00, 0x3c, 0x00, 0xff, 0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0x38, 0x00, 0x3f,
    0x00, 0x3f, 0x00, 0x38, 0x00, 0xff, 0x00, 0xff, 0x80, 0x00, 0xe0, 0x00, 0xe0, 0x00, 0x80, 0x00,
    0x80, 0x00, 0xe0, 0x00, 0xe0, 0x00, 0x80, 0xf8, 0x00, 0xfc, 0x00, 0x3e, 0x00, 0x3f, 0x00, 0x3b,
    0x00, 0x39, 0x00, 0xf8, 0x00, 0xf8, 0x03, 0x00, 0x07, 0x00, 0x0f, 0x00, 0xbf, 0x00, 0xfb, 0x00,
    0xf3, 0x00, 0xe3, 0x00, 0x43, 0xe0, 0x00, 0xe0, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80,
    0x00, 0xe0, 0x00, 0xe0,
];

fn main() {
    // Start our program counter where the program begins.
    let mut pc = PROGRAM_OFFSET;
    // Initialize our memory, registers, and frame buffer.
    let mut memory = Memory::new();
    let mut registers = Registers::new();
    let mut frame_buffer = vec![false; WIDTH * HEIGHT];

    // Load the IBM logo ROM into memory, starting at PROGRAM_OFFSET.
    for (address, value) in IBM_LOGO_ROM.iter().enumerate() {
        memory.write(address as u16 + PROGRAM_OFFSET, *value);
    }

    // Fetch, decode, execute loop.
    loop {
        // Fetch the next instruction (2 bytes).
        let instruction = fetch(&memory, pc);
        // Decode the instruction into our Opcode enum.
        let opcode = decode(instruction);
        // Execute the instruction
        execute(
            opcode,
            &mut memory,
            &mut registers,
            &mut frame_buffer,
            &mut pc,
        );

        // Once we hit PROGRAM_OFFSET + 0x28 we're about to decode a JP instruction that signals
        // the IBM logo being fully drawn. We break out of the loop here and then render an image
        // from the frame buffer.
        if pc == PROGRAM_OFFSET + 0x28 {
            break;
        }
    }

    // Render an image from the frame buffer.
    render(&frame_buffer);
}

fn fetch(memory: &Memory, pc: u16) -> u16 {
    // Most significant byte first
    (memory.read(pc) as u16) << 8 | memory.read(pc + 1) as u16
}

fn decode(instruction: u16) -> Opcode {
    // Separate out all the different parts from the raw instruction. This is doing extra work
    // because not all of the parts are needed by each instruction but it makes the decoding easy
    // to understand.

    // The last 3 nibbles
    // 0000XXXX XXXXXXXX
    let nnn = Nnn(instruction & 0x0fff);
    // The last byte
    // 00000000 XXXXXXXX
    let kk = Kk((instruction & 0x00ff) as u8);
    // Register x
    // 0000XXXX 00000000
    let x = Register(((instruction >> 8) & 0xf) as u8);
    // Register y
    // 00000000 XXXX0000
    let y = Register(((instruction >> 4) & 0xf) as u8);
    // The last nibble
    // 00000000 0000XXXX
    let n = N((instruction & 0x000f) as u8);

    // Match on the first nibble of the raw instruction
    match instruction >> 12 {
        0x0 => match instruction {
            0x00e0 => Opcode::CLS,
            // There are other instructions that start with 0x0, but we only need this one.
            _ => unimplemented!("{:#06x}", instruction),
        },
        0x6 => Opcode::LD6(x, kk),
        0x7 => Opcode::ADD(x, kk),
        0xa => Opcode::LDI(nnn),
        0xd => Opcode::DRW(x, y, n),
        // There are other instructions but we don't need them to render the IBM logo.
        _ => unimplemented!("{:#x}", instruction >> 12),
    }
}

fn execute(
    opcode: Opcode,
    memory: &mut Memory,
    registers: &mut Registers,
    frame_buffer: &mut [bool],
    pc: &mut u16,
) {
    match opcode {
        // 00E0 - CLS
        //
        // Clear the display.
        Opcode::CLS => {
            // NOTE: We don't need to do anything here as we initialze an empty frame buffer and
            // we only draw a single frame, so we never need to clear it.
        }

        // 6xkk - LD Vx, byte
        //
        // Set Vx = kk.
        // The interpreter puts the value kk into register Vx.
        Opcode::LD6(register, Kk(value)) => registers.write(&register, value),

        // Annn - LD I, addr
        //
        // Set I = nnn.
        // The value of register I is set to nnn.
        Opcode::LDI(Nnn(value)) => registers.i = value,

        // Dxyn - DRW Vx, Vy, nibble
        //
        // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
        // The interpreter reads n bytes from memory, starting at the address stored in I. These
        // bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed
        // onto the existing screen. If this causes any pixels to be erased, VF is set to 1,
        // otherwise it is set to 0. If the sprite is positioned so part of it is outside the
        // coordinates of the display, it wraps around to the opposite side of the screen.
        Opcode::DRW(x_register, y_register, N(height)) => {
            let x_offset = registers.read(&x_register);
            let y_offset = registers.read(&y_register);

            for y in 0..height {
                let line = memory.read(registers.i + y as u16);
                for x in 0..8 {
                    if (line & (0x80 >> x)) != 0 {
                        // NOTE: Not handling wrapping
                        let x = x_offset + x;
                        let y = y_offset + y;

                        // NOTE: Not handling collision detection
                        let l = y as usize * WIDTH + x as usize;
                        assert!(l < WIDTH * HEIGHT);
                        frame_buffer[l] = !frame_buffer[l];
                    }
                }
            }
        }

        // 7xkk - ADD Vx, byte
        //
        // Set Vx = Vx + kk.
        // Adds the value kk to the value of register Vx, then stores the result in Vx.
        Opcode::ADD(x, Kk(kk)) => {
            let result = registers.read(&x).wrapping_add(kk);
            registers.write(&x, result);
        }
    }

    // Increment our program counter.
    // NOTE: None of our instructions use the program counter so we're safe to simply increment by
    // 2 each loop. This isn't true for all instructions.
    *pc += 2;
}

#[derive(Debug)]
enum Opcode {
    CLS,
    LDI(Nnn),
    LD6(Register, Kk),
    ADD(Register, Kk),
    DRW(Register, Register, N),
}

#[derive(Debug)]
struct Register(u8);

#[derive(Debug)]
struct Nnn(u16);

#[derive(Debug)]
struct Kk(u8);

#[derive(Debug)]
struct N(u8);

struct Registers {
    registers: [u8; 16],
    i: u16,
}

impl Registers {
    fn new() -> Self {
        Self {
            registers: [0; 16],
            i: 0,
        }
    }

    fn read(&self, register: &Register) -> u8 {
        // Ensure we're referencing a valid register
        assert!(register.0 <= 0xf);
        self.registers[register.0 as usize]
    }

    fn write(&mut self, register: &Register, value: u8) {
        // Ensure we're referencing a valid register
        assert!(register.0 <= 0xf);
        self.registers[register.0 as usize] = value
    }
}

struct Memory {
    memory: [u8; MEMORY_LENGTH],
}

impl Memory {
    fn new() -> Self {
        Self {
            memory: [0; MEMORY_LENGTH],
        }
    }

    fn read(&self, address: u16) -> u8 {
        if address < 80 {
            // A system font lives at this address
            unimplemented!("font");
        } else if address < PROGRAM_OFFSET {
            // This is where the original intepreter would live
            panic!("invalid memory access");
        } else {
            self.memory[address as usize]
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}

// https://en.wikipedia.org/wiki/Netpbm_format#PBM_example
fn render(frame_buffer: &[bool]) {
    // Print the PBM header
    print!("P1\n{} {}", WIDTH, HEIGHT);
    let mut i = 0;
    for pixel in frame_buffer {
        // Add a line break every WIDTH pixels, this isn't required
        // by the PBM format but makes the file human readable.
        if i % WIDTH == 0 {
            println!();
        }
        // If the pixel is on, print a 1, else a 0.
        if *pixel {
            print!("1 ")
        } else {
            print!("0 ")
        }
        i += 1;
    }
    println!();
}
