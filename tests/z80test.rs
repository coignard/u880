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

const Z80FULL: &[u8] = include_bytes!("assets/z80full.out");
const START_ADDR: u16 = 0x8000;

// These should fail.
#[rustfmt::skip]
const COMMON_EXPECTED: &[&str] = &[
    "089 LDIR->NOP'  FAILED",           // U880: WZ differs after ED 00 abort
    "CRC:9DC743B5   Expected:CC93B5EC",
    "090 LDDR->NOP'  FAILED",           // U880: WZ differs after ED 00 abort
    "CRC:9C1DEA50   Expected:CD491C09",
    "098 INI  FAILED",                  // U880: CF unaffected
    "CRC:0873E884   Expected:03DA7534",
    "099 IND  FAILED",                  // U880: CF unaffected
    "CRC:4799F637   Expected:4C306B87",
    "100 INIR  FAILED",                 // U880: CF unaffected
    "CRC:5291BB41   Expected:B1C580A1",
    "101 INDR  FAILED",                 // U880: CF unaffected
    "CRC:6BA43F4A   Expected:88F004AA",
    "102 INIR->NOP'  FAILED",           // U880: CF unaffected
    "CRC:E97393FA   Expected:454E3531",
    "103 INDR->NOP'  FAILED",           // U880: CF unaffected
    "CRC:AA81E60F   Expected:06BC40C4",
    "107 OUTI  FAILED",                 // U880: CF unaffected
    "CRC:6AA4B16F   Expected:6B09C8E2",
    "108 OUTD  FAILED",                 // U880: CF unaffected
    "CRC:C02B94F2   Expected:C186ED7F",
    "109 OTIR  FAILED",                 // U880: CF unaffected
    "CRC:2DC02583   Expected:366E1554",
    "110 OTDR  FAILED",                 // U880: CF unaffected
    "CRC:CA1B9C47   Expected:1781B976",
];

fn run_z80_test(rom: &[u8], revision: Revision, expected: &[&str]) {
    let mut cpu = Cpu::with_revision(revision);
    let mut mem = vec![0u8; 1 << 16];
    let mut bus;

    mem[START_ADDR as usize..START_ADDR as usize + rom.len()].copy_from_slice(rom);
    mem[0x1601] = 0xC9;
    mem[0x0010] = 0xC9;

    bus = cpu.prefetch(START_ADDR);

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
        } else if bus & pins::IORQ != 0 {
            if bus & pins::RD != 0 {
                bus = pins::set_data(bus, 0xBF);
            }
        }

        if cpu.opdone() {
            let pc = pins::addr(bus);

            if pc == 0x0000 {
                let _ = io::stdout().write_all(b"\n");
                break;
            } else if pc == 0x0010 {
                let mut ch = cpu.regs.a as char;
                if ch == '\r' {
                    ch = '\n';
                } else if ch as u8 == 23 || ch as u8 == 26 {
                    ch = ' ';
                }

                print!("{}", ch);
                let _ = io::stdout().flush();

                if ch == '\n' || (ch >= ' ' && ch <= '~') {
                    output.push(ch);
                }
            }
        }
    }

    if let Some(missing) = expected.iter().find(|&&s| !output.contains(s)) {
        panic!(
            "log mismatch for revision '{:?}': expected line not found: expected='{}'",
            revision, missing
        );
    }
}

#[test]
#[ignore]
fn test_z80full() {
    #[rustfmt::skip]
    let mut older_expected = vec![
        "003 SCF (NEC) OK",                 // U880: XF/YF = A
        "004 CCF (NEC) OK",                 // U880: XF/YF = A
        "001 SCF  FAILED",                  // U880: XF/YF = A
        "CRC:45FC79B5   Expected:D841BD8A",
        "002 CCF  FAILED",                  // U880: XF/YF = A
        "CRC:A206B5E3   Expected:3FBB71DC",
        "005 SCF (ST)  FAILED",             // ST CMOS not emulated
        "CRC:45FC79B5   Expected:58E950E4",
        "006 CCF (ST)  FAILED",             // ST CMOS not emulated
        "CRC:A206B5E3   Expected:BF139CB2",
        "Result: 016 of 160 tests failed.",
    ];
    older_expected.extend_from_slice(COMMON_EXPECTED);

    run_z80_test(Z80FULL, Revision::Older, &older_expected);

    #[rustfmt::skip]
    let mut newer_expected = vec![
        "003 SCF (NEC) Skipped",            // U880: XF/YF = A|F_in, coincides on initial vector
        "004 CCF (NEC) Skipped",            // U880: XF/YF = A|F_in, coincides on initial vector
        "007 SCF+CCF  FAILED",              // U880: XF/YF = A|F_in
        "CRC:0D3B8D53   Expected:9086496C",
        "008 CCF+SCF  FAILED",              // U880 Newer: XF/YF = A|F_in
        "CRC:D841BD8A   Expected:45FC79B5",
        "Result: 014 of 160 tests failed.",
    ];
    newer_expected.extend_from_slice(COMMON_EXPECTED);

    run_z80_test(Z80FULL, Revision::Newer, &newer_expected);
}
