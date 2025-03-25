// Code inspired by https://github.com/emk/toyos-rs/blob/master/crates/pic8259_simple/src/lib.rs
// for information of PIC https://wiki.osdev.org/8259_PIC

use x86_64::instructions::port::Port;

const PIC_EOI: u8 = 0x20;
const CMD_INIT: u8 = 0x11;
const MODE_8086: u8 = 0x01;

struct Pic {
    offset: u8,
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.offset <= interrupt_id && interrupt_id < self.offset + 8
    }

    pub unsafe fn end_of_interrupt(&mut self) {
        self.command.write(PIC_EOI);
    }
}

pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const unsafe fn new(offset1: u8, offset2: u8) -> ChainedPics {
        ChainedPics {
            pics: [
                Pic {
                    offset: offset1,
                    command: Port::new(0x20),
                    data: Port::new(0x21),
                },
                Pic {
                    offset: offset2,
                    command: Port::new(0xA0),
                    data: Port::new(0xA1),
                },
            ],
        }
    }

    pub unsafe fn initialize(&mut self) {
        let mut wait_port: Port<u8> = Port::new(0x80);
        let mut wait = || wait_port.write(0);

        let saved_mask_1 = self.pics[0].data.read();
        let saved_mask_2 = self.pics[1].data.read();

        self.pics[0].command.write(CMD_INIT);
        wait();
        self.pics[1].command.write(CMD_INIT);
        wait();
        self.pics[0].data.write(self.pics[0].offset);
        wait();
        self.pics[1].data.write(self.pics[1].offset);
        wait();

        self.pics[0].data.write(MODE_8086);
        wait();
        self.pics[1].data.write(MODE_8086);
        wait();

        self.pics[0].data.write(saved_mask_1);
        self.pics[1].data.write(saved_mask_2);
    }

    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(interrupt_id))
    }

    pub unsafe fn send_eoi(&mut self, interrupt_id: u8) {
        if self.handles_interrupt(interrupt_id) {
            if self.pics[1].handles_interrupt(interrupt_id) {
                self.pics[1].end_of_interrupt();
            }
            self.pics[0].end_of_interrupt();
        }
    }
}
