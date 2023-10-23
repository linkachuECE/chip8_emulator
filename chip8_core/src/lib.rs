use rand::Rng;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const FONTSET_SIZE: usize = 80;
const FONT_SIZE: usize = 5;
const FONTSET_ADDR: usize = 0x0;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

// Main class for the emulator
pub struct Emu {
    pc: u16,                                        // Program counter
    ram: [u8; RAM_SIZE],                            // RAM, 4KB long
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],   // Array of black-and-white pixels
    v_reg: [u8; NUM_REGS],                          // V register
    i_reg: u16,                                     // I register
    sp: u16,                                        // Stack pointer
    stack: [u16; STACK_SIZE],                       // Stack
    keys: [bool; NUM_KEYS],                         // Holds the state of each key
    dt: u8,                                         // Delay timer
    st: u8                                          // Sound timer
}

pub const START_ADDR: u16 = 0x200;

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn get_display(&self) -> &[bool]{
        &self.screen
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn keypress(&mut self, key: usize, pressed: bool){
        self.keys[key] = pressed;
    }

    pub fn push(&mut self, val: u16){
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn reset(&mut self){
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
    }

    pub fn tick(&mut self){
        // Fetch
        let op = self.fetch();
        
        // debug_println!("Executing opcode: {:#06x}", op);

        // Decode and execute
        self.execute(op);
    }

    fn execute(&mut self, op: u16){
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match(digit1, digit2, digit3, digit4) {
            // 0x0000: No operation (NOP)
            (0,0,0,0) => return,

            // 0x00E0: (CLS)
            // Clear screen 
            (0,0,0xE,0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },

            // 0x00EE: (RET)
            // Return from subroutine
            (0,0,0xE,0xE) => {
                // Pop the return address from the stack change the program counter value
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },

            // 0x1NNN: (JP addr)
            // Jump
            (1,_,_,_) => {
                // Get the new address from the bottom three bytes
                let nnn = op & 0x0FFF;

                // Jump to the new address
                self.pc = nnn;
            },

            // 0x2NNN: (CALL addr)
            // Call Subroutine 
            (2,_,_,_) => {
                self.push(self.pc);
                self.pc = op & 0x0FFF;
            },

            // 0x3XNN: (SE Vx, byte)
            // Skip next if Vx == NN 
            (3,_,_,_) => {
                let x = digit2 as usize;

                let nn = (op & 0x00FF) as u8;

                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },

            // 0x4XNN: (SNE Vx, byte)
            // Skip next if Vx != NN 
            (4,_,_,_) => {
                let x = digit2 as usize;

                let nn = (op & 0x00FF) as u8;

                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },

            // 0x5XY0: (SE Vx, Vy)
            // Skip next if Vx == Vy 
            (5,_,_,_) => {
                let x = digit2 as usize;
                let y = digit2 as usize;

                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },

            // 0x6XNN: (LD Vx, byte)
            // Set Vx = NN 
            (6,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            },

            // 0x7XNN: (ADD Vx, byte)
            // Set Vx += NN 
            (7,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },

            // 0x8XY0: (LD Vx, Vy)
            // Set Vx = Vy 
            (8,_,_,0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },

            // 0x8XY1: (OR Vx, Vy)
            // Set Vx = Vx OR Vy 
            (8,_,_,1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_reg[x] |= self.v_reg[y];
            },

            // 0x8XY2: (AND Vx, Vy)
            // Set Vx = Vx AND Vy 
            (8,_,_,2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_reg[x] &= self.v_reg[y];
            },

            // 0x8XY3: (XOR Vx, Vy)
            // Set Vx = Vx XOR Vy
            (8,_,_,3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_reg[x] ^= self.v_reg[y];
            },

            // 0x8XY4: (ADD Vx, Vy)
            // Set Vx = Vx + Vy, set VF = carry 
            (8,_,_,4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (sum, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                
                self.v_reg[x] = sum;
                self.v_reg[0xF] = carry as u8;
            },

            // 0x8XY5: (SUB Vx, Vy)
            // Set Vx = Vy - Vy, set VF = NOT borrow 
            (8,_,_,5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let not_borrow: bool = self.v_reg[x] > self.v_reg[y];
                let difference = self.v_reg[x].wrapping_sub(self.v_reg[y]);

                self.v_reg[x] = difference;
                self.v_reg[0xF] = not_borrow as u8;
            },

            // 0x8XY6: (SHR Vx {, Vy})
            // SHR = Single right-shift
            // Set Vx = Vx >> 1, VF = LSB before shift
            (8,_,_,6) => {
                let x = digit2 as usize;

                let lsb = self.v_reg[x] & 0x01;

                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },

            // 0x8XY7: (SUBN Vx, Vy)
            // Set Vx = Vy - Vx, Set VF = NOT borrow
            (8,_,_,7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let not_borrow: bool = self.v_reg[y] > self.v_reg[x];

                self.v_reg[x] = self.v_reg[y].wrapping_sub(self.v_reg[x]);
                self.v_reg[0xF] = not_borrow as u8;
            },

            // 0x8XYE: (SHL Vx {, Vy})
            // Set Vx = Vx << 1, VF = MSB before shift
            (8,_,_,0xE) => {
                let x = digit2 as usize;

                let msb: u8 = (self.v_reg[x] >> 7) & 0x01;

                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },

            // 0x9XY0: (SNE Vx, Vy)
            // Skip next instruction if Vx != Vy
            (9,_,_,0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },

            // 0xANNN: (LD I, addr)
            // Set I = NNN
            (0xA,_,_,_) => {
                self.i_reg = op & 0x0FFF;
            },

            // 0xBNNN: (JP V0, addr)
            // Jump to location NNN + V0
            (0xB,_,_,_) => {
                self.pc = self.v_reg[0] as u16 + (op & 0x0FFF);
            },

            // 0xCXNN: RND Vx, byte
            // Set Vx = random byte AND NN
            (0xC,_,_,_) => {
                let x = digit2 as usize;
                
                let mut rng = rand::thread_rng();
                let rand: u8 = rng.gen();

                self.v_reg[x] = rand & (op & 0x00FF) as u8;
            },

            // 0xDXYN: DRW Vx, Vy, nibble
            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            (0xD,_,_,_) => {
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;

                // The last digit determines how many rows high our sprite is
                let num_rows = digit4;
                
                // Keep track if any pixels were flipped
                let mut flipped = false;
                
                // Iterate over each row of our sprite
                for y_line in 0..num_rows {
                    // Determine which memory address our row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    // Iterate over each column in our row
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // Get our pixel's index for our 1D screen array
                            let idx = x + SCREEN_WIDTH * y;

                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                // Populate VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }

            },

            // 0xEX9E: SKP Vx
            // Skip next instruction if key with the value of Vx is pressed
            (0xE,_,0x9,0xE) => {
                let x = digit2 as usize;

                let key_no = self.v_reg[x] as usize;

                if self.keys[key_no] {
                    self.pc += 2;
                }
            },

            // 0xEXA1: SKNP Vx
            // Skip next instruction if key with the value of Vx is not pressed.
            (0xE,_,0xA,0x1) => {
                let x = digit2 as usize;

                let key_no = self.v_reg[x] as usize;

                if !self.keys[key_no] {
                    self.pc += 2;
                }
            },

            // 0xFX07: LD Vx, DT
            // Set Vx = delay timer value.
            (0xF,_,0x0,0x7) => {
                let x = digit2 as usize;

                self.v_reg[x] = self.dt;
            },

            // 0xFX0A: LD Vx, K
            // Wait for a key press, store the value of the key in Vx.
            (0xF,_,0x0,0xA) => {
                let x = digit2 as usize;

                // Initialize this to an invalid number initially
                let mut key_no = NUM_KEYS as u8;

                // Check if any of the keys have been pressed
                for i in 0..NUM_KEYS {
                    if self.keys[i] {
                        key_no = i as u8;
                    }
                }

                // If none have been pressed, repeat the same instruction
                if key_no == NUM_KEYS as u8 {
                    self.pc -= 2;
                } else {
                    self.v_reg[x] = key_no;
                }
            },

            // 0xFX15: LD DT, Vx
            // Set delay timer = Vx
            (0xF,_,0x1,0x5) => {
                let x = digit2 as usize;

                self.dt = self.v_reg[x];
            },

            // 0xFX18: LD ST, Vx
            // Set sound timer = Vx
            (0xF,_,0x1,0x8) => {
                let x = digit2 as usize;

                self.st = self.v_reg[x];
            },

            // 0xFX1E: ADD I, Vx
            // Set I = I + Vx
            (0xF,_,0x1,0xE) => {
                let x = digit2 as usize;

                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
            },

            // 0xFX29: LD F, Vx
            // Set I = location of sprite for digit Vx
            (0xF,_,0x2,0x9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;

                self.i_reg = FONTSET_ADDR as u16 + (FONT_SIZE as u16 * c);
            },

            // 0xFX33: LD B, Vx
            // Store BCD representation of Vx in memory locations I, I+1, and I+2.
            (0xF,_,0x3,0x3) => {
                let x = digit2 as usize;

                let hundreds = (x / 100) as u8;
                let tens = ((x % 100) / 10) as u8;
                let ones = (x % 10) as u8;

                let start_addr = self.i_reg as usize;

                self.ram[start_addr] = hundreds;
                self.ram[start_addr + 1] = tens;
                self.ram[start_addr + 2] = ones;
            },

            // 0xFX55: LD [I], Vx
            // Store registers V0 through Vx in memory starting at location I.
            (0xF,_,0x5,0x5) => {
                let x = digit2 as usize;
                let start_addr = self.i_reg as usize;

                for i in 0..x {
                    self.ram[start_addr + i] = self.v_reg[i];
                }
            },

            // 0xFX65: LD Vx, [I]
            // Read registers V0 through Vx from memory starting at location I.
            (0xF,_,0x6,0x5) => {
                let x = digit2 as usize;
                let start_addr = self.i_reg as usize;

                for i in 0..x {
                    self.v_reg[i] = self.ram[start_addr + i];
                }
            }


            (_,_,_,_) => unimplemented!("Unimplented opcode: {:#04x}", op)
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[(self.pc) as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;

        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;

        op
    }

    pub fn tick_timers(&mut self){
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }

            self.st -= 1;
        }
    }

}