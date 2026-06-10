// This file is part of u880.
//
// Copyright (c) 2026  René Coignard <contact@renecoignard.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::io::{self, Write};
use u880::{Cpu, Revision, pins};

const CPUTEST: &[u8] = include_bytes!("assets/cputest.com");

fn run_cpu_test(rom: &[u8], revision: Revision) {
    let mut cpu = Cpu::with_revision(revision);
    let mut mem = vec![0u8; 1 << 16];
    let mut bus;

    mem[0x0100..0x0100 + rom.len()].copy_from_slice(rom);
    mem[0x0005] = 0xC9;

    bus = cpu.prefetch(0x0100);

    let mut output = String::new();

    loop {
        bus = cpu.tick(bus);

        if bus & pins::MREQ != 0 {
            let addr = pins::addr(bus) as usize;
            if bus & pins::RD != 0 {
                bus = pins::set_data(bus, mem[addr]);
            } else if bus & pins::WR != 0 {
                mem[addr] = pins::data(bus);
            }
        }

        if cpu.opdone() {
            let pc = pins::addr(bus);

            if pc == 0x0000 {
                let _ = io::stdout().write_all(b"\n");
                break;
            } else if pc == 0x0005 {
                match cpu.regs.c {
                    2 => {
                        let ch = cpu.regs.e as char;
                        print!("{}", ch);
                        let _ = io::stdout().flush();
                        output.push(ch);
                    }
                    9 => {
                        let mut addr = cpu.regs.de() as usize;
                        let mut msg = String::new();

                        loop {
                            let ch = mem[addr] as char;
                            if ch == '$' {
                                break;
                            }
                            if ch != '\r' {
                                msg.push(ch);
                            }
                            addr = (addr + 1) & 0xFFFF;
                        }

                        print!("{}", msg);
                        let _ = io::stdout().flush();
                        output.push_str(&msg);
                    }
                    _ => panic!(),
                }
            }
        }
    }

    assert!(output.contains("CPU TESTS OK"));
}

#[test]
#[ignore]
fn test_cpu_test() {
    run_cpu_test(CPUTEST, Revision::Older);
    run_cpu_test(CPUTEST, Revision::Newer);
}
