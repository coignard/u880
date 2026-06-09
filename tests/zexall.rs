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

const ZEXALL: &[u8] = include_bytes!("assets/zexall.com");
const ZEXDOC: &[u8] = include_bytes!("assets/zexdoc.com");

fn run_zex_test(rom: &[u8], revision: Revision) {
    let mut cpu = Cpu::with_revision(revision);
    let mut mem = vec![0u8; 1 << 16];
    let mut bus;

    mem[0x0100..0x0100 + rom.len()].copy_from_slice(rom);
    mem[0x0005] = 0xC9;

    bus = cpu.prefetch(0x0100);

    let mut tests_passed = 0;

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
                        print!("{}", cpu.regs.e as char);
                        let _ = io::stdout().flush();
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

                        if msg.contains("OK") {
                            tests_passed += 1;
                        }
                    }
                    _ => panic!(),
                }
            }
        }
    }

    assert_eq!(67, tests_passed);
}

#[test]
#[ignore]
fn test_zexdoc() {
    run_zex_test(ZEXDOC, Revision::Older);
    run_zex_test(ZEXDOC, Revision::Newer);
}

#[test]
#[ignore]
fn test_zexall() {
    run_zex_test(ZEXALL, Revision::Older);
    run_zex_test(ZEXALL, Revision::Newer);
}
