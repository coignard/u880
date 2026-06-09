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

use crate::cpu::Cpu;
use crate::flags::Flags;
use crate::pins;

const INDIRECT_BYTES: &[u8] = &[
    0x34, 0x35, 0x36, 0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77,
    0x7E, 0x86, 0x8E, 0x96, 0x9E, 0xA6, 0xAE, 0xB6, 0xBE,
];

const fn build_lookup(bytes: &[u8]) -> [bool; 256] {
    let mut table = [false; 256];
    let mut i = 0;
    while i < bytes.len() {
        table[bytes[i] as usize] = true;
        i += 1;
    }
    table
}

pub const INDIRECT_TABLE: [bool; 256] = build_lookup(INDIRECT_BYTES);

#[inline(always)]
pub fn decode(cpu: &mut Cpu, mut pins: u64) -> u64 {
    macro_rules! goto {
        ($step:expr) => {{
            cpu.state.step = $step;
            return pins;
        }};
    }
    macro_rules! fetch {
        () => {
            return cpu.fetch(pins);
        };
    }
    macro_rules! fetch_dd {
        () => {
            return cpu.fetch_dd(pins);
        };
    }
    macro_rules! fetch_fd {
        () => {
            return cpu.fetch_fd(pins);
        };
    }
    macro_rules! fetch_ed {
        () => {
            return cpu.fetch_ed(pins);
        };
    }
    macro_rules! fetch_cb {
        () => {
            return cpu.fetch_cb(pins);
        };
    }
    macro_rules! wait {
        () => {
            if pins & pins::WAIT != 0 {
                return pins;
            }
        };
    }

    macro_rules! mread {
        ($addr:expr) => {
            pins = pins::set_addr_ctrl(pins, $addr, pins::MREQ | pins::RD);
        };
    }
    macro_rules! mwrite {
        ($addr:expr, $val:expr) => {
            pins = pins::set_addr_data_ctrl(pins, $addr, $val, pins::MREQ | pins::WR);
        };
    }
    macro_rules! ioread {
        ($addr:expr) => {
            pins = pins::set_addr_ctrl(pins, $addr, pins::IORQ | pins::RD);
        };
    }
    macro_rules! iowrite {
        ($addr:expr, $val:expr) => {
            pins = pins::set_addr_data_ctrl(pins, $addr, $val, pins::IORQ | pins::WR);
        };
    }
    macro_rules! gd {
        () => {
            pins::data(pins)
        };
    }

    macro_rules! cc_nz {
        () => {
            !cpu.regs.f.contains(Flags::Z)
        };
    }
    macro_rules! cc_z {
        () => {
            cpu.regs.f.contains(Flags::Z)
        };
    }
    macro_rules! cc_nc {
        () => {
            !cpu.regs.f.contains(Flags::C)
        };
    }
    macro_rules! cc_c {
        () => {
            cpu.regs.f.contains(Flags::C)
        };
    }
    macro_rules! cc_po {
        () => {
            !cpu.regs.f.contains(Flags::P)
        };
    }
    macro_rules! cc_pe {
        () => {
            cpu.regs.f.contains(Flags::P)
        };
    }
    macro_rules! cc_p {
        () => {
            !cpu.regs.f.contains(Flags::S)
        };
    }
    macro_rules! cc_m {
        () => {
            cpu.regs.f.contains(Flags::S)
        };
    }

    macro_rules! pc_postinc {
        () => {{
            let a = cpu.regs.pc;
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            a
        }};
    }
    macro_rules! sp_predec {
        () => {{
            cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
            cpu.regs.sp
        }};
    }
    macro_rules! sp_postinc {
        () => {{
            let a = cpu.regs.sp;
            cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
            a
        }};
    }
    macro_rules! wz_postinc {
        () => {{
            let a = cpu.regs.wz;
            cpu.regs.wz = cpu.regs.wz.wrapping_add(1);
            a
        }};
    }
    macro_rules! hl_postinc {
        () => {{
            let a = cpu.regs.hl();
            cpu.regs.set_hl(a.wrapping_add(1));
            a
        }};
    }
    macro_rules! de_postinc {
        () => {{
            let a = cpu.regs.de();
            cpu.regs.set_de(a.wrapping_add(1));
            a
        }};
    }
    macro_rules! hl_postdec {
        () => {{
            let a = cpu.regs.hl();
            cpu.regs.set_hl(a.wrapping_sub(1));
            a
        }};
    }
    macro_rules! de_postdec {
        () => {{
            let a = cpu.regs.de();
            cpu.regs.set_de(a.wrapping_sub(1));
            a
        }};
    }

    macro_rules! inc8 {
        ($reg:expr) => {{
            let (r, f) = crate::alu::inc8($reg, cpu.regs.f);
            cpu.regs.f = f;
            r
        }};
    }
    macro_rules! dec8 {
        ($reg:expr) => {{
            let (r, f) = crate::alu::dec8($reg, cpu.regs.f);
            cpu.regs.f = f;
            r
        }};
    }
    macro_rules! add8 {
        ($val:expr) => {{
            let (r, f) = crate::alu::add8(cpu.regs.a, $val, false);
            cpu.regs.f = f;
            cpu.regs.a = r;
        }};
    }
    macro_rules! adc8 {
        ($val:expr) => {{
            let (r, f) = crate::alu::add8(cpu.regs.a, $val, cpu.regs.f.contains(Flags::C));
            cpu.regs.f = f;
            cpu.regs.a = r;
        }};
    }
    macro_rules! sub8 {
        ($val:expr) => {{
            let (r, f) = crate::alu::sub8(cpu.regs.a, $val, false);
            cpu.regs.f = f;
            cpu.regs.a = r;
        }};
    }
    macro_rules! sbc8 {
        ($val:expr) => {{
            let (r, f) = crate::alu::sub8(cpu.regs.a, $val, cpu.regs.f.contains(Flags::C));
            cpu.regs.f = f;
            cpu.regs.a = r;
        }};
    }
    macro_rules! and8 {
        ($val:expr) => {{
            let (r, f) = crate::alu::and8(cpu.regs.a, $val);
            cpu.regs.f = f;
            cpu.regs.a = r;
        }};
    }
    macro_rules! xor8 {
        ($val:expr) => {{
            let (r, f) = crate::alu::xor8(cpu.regs.a, $val);
            cpu.regs.f = f;
            cpu.regs.a = r;
        }};
    }
    macro_rules! or8 {
        ($val:expr) => {{
            let (r, f) = crate::alu::or8(cpu.regs.a, $val);
            cpu.regs.f = f;
            cpu.regs.a = r;
        }};
    }
    macro_rules! cp8 {
        ($val:expr) => {{
            cpu.regs.f = crate::alu::cp8(cpu.regs.a, $val);
        }};
    }
    macro_rules! neg8 {
        () => {{
            let (r, f) = crate::alu::neg8(cpu.regs.a);
            cpu.regs.f = f;
            cpu.regs.a = r;
        }};
    }

    macro_rules! rlca {
        () => {
            let a = cpu.regs.a;
            let res = a.rotate_left(1);
            cpu.regs.f =
                Flags::from_bits_retain(((a >> 7) & 1) | (cpu.regs.f.bits() & 0xC4) | (res & 0x28));
            cpu.regs.a = res;
        };
    }
    macro_rules! rrca {
        () => {
            let a = cpu.regs.a;
            let res = a.rotate_right(1);
            cpu.regs.f =
                Flags::from_bits_retain((a & 1) | (cpu.regs.f.bits() & 0xC4) | (res & 0x28));
            cpu.regs.a = res;
        };
    }
    macro_rules! rla {
        () => {
            let a = cpu.regs.a;
            let res = (a << 1) | (if cpu.regs.f.contains(Flags::C) { 1 } else { 0 });
            cpu.regs.f =
                Flags::from_bits_retain(((a >> 7) & 1) | (cpu.regs.f.bits() & 0xC4) | (res & 0x28));
            cpu.regs.a = res;
        };
    }
    macro_rules! rra {
        () => {
            let a = cpu.regs.a;
            let res = (a >> 1)
                | (if cpu.regs.f.contains(Flags::C) {
                    0x80
                } else {
                    0
                });
            cpu.regs.f =
                Flags::from_bits_retain((a & 1) | (cpu.regs.f.bits() & 0xC4) | (res & 0x28));
            cpu.regs.a = res;
        };
    }

    macro_rules! in8 {
        ($val:expr) => {{
            cpu.regs.f = Flags::from_bits_retain(
                (cpu.regs.f.bits() & Flags::C) | crate::alu::szp_flags($val).bits(),
            );
            $val
        }};
    }

    match cpu.state.step {
        0 => {
            fetch!();
        }
        1 => {
            goto!(512);
        }
        2 => {
            goto!(518);
        }
        3 => {
            let v = cpu.regs.bc().wrapping_add(1);
            cpu.regs.set_bc(v);
            goto!(521);
        }
        4 => {
            cpu.regs.b = inc8!(cpu.regs.b);
            fetch!();
        }
        5 => {
            cpu.regs.b = dec8!(cpu.regs.b);
            fetch!();
        }
        6 => {
            goto!(523);
        }
        7 => {
            rlca!();
            fetch!();
        }
        8 => {
            cpu.regs.swap_af();
            fetch!();
        }
        9 => {
            crate::alu::add16(cpu, cpu.regs.bc());
            goto!(526);
        }
        10 => {
            goto!(533);
        }
        11 => {
            let v = cpu.regs.bc().wrapping_sub(1);
            cpu.regs.set_bc(v);
            goto!(536);
        }
        12 => {
            cpu.regs.c = inc8!(cpu.regs.c);
            fetch!();
        }
        13 => {
            cpu.regs.c = dec8!(cpu.regs.c);
            fetch!();
        }
        14 => {
            goto!(538);
        }
        15 => {
            rrca!();
            fetch!();
        }
        16 => {
            goto!(541);
        }
        17 => {
            goto!(550);
        }
        18 => {
            goto!(556);
        }
        19 => {
            let v = cpu.regs.de().wrapping_add(1);
            cpu.regs.set_de(v);
            goto!(559);
        }
        20 => {
            cpu.regs.d = inc8!(cpu.regs.d);
            fetch!();
        }
        21 => {
            cpu.regs.d = dec8!(cpu.regs.d);
            fetch!();
        }
        22 => {
            goto!(561);
        }
        23 => {
            rla!();
            fetch!();
        }
        24 => {
            goto!(564);
        }
        25 => {
            crate::alu::add16(cpu, cpu.regs.de());
            goto!(572);
        }
        26 => {
            goto!(579);
        }
        27 => {
            let v = cpu.regs.de().wrapping_sub(1);
            cpu.regs.set_de(v);
            goto!(582);
        }
        28 => {
            cpu.regs.e = inc8!(cpu.regs.e);
            fetch!();
        }
        29 => {
            cpu.regs.e = dec8!(cpu.regs.e);
            fetch!();
        }
        30 => {
            goto!(584);
        }
        31 => {
            rra!();
            fetch!();
        }
        32 => {
            goto!(587);
        }
        33 => {
            goto!(595);
        }
        34 => {
            goto!(601);
        }
        35 => {
            let v = cpu.regs.hlx(cpu.state.hlx_idx).wrapping_add(1);
            cpu.regs.set_hlx(cpu.state.hlx_idx, v);
            goto!(613);
        }
        36 => {
            let v = inc8!(cpu.regs.hlx_h(cpu.state.hlx_idx));
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, v);
            fetch!();
        }
        37 => {
            let v = dec8!(cpu.regs.hlx_h(cpu.state.hlx_idx));
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, v);
            fetch!();
        }
        38 => {
            goto!(615);
        }
        39 => {
            let (r, f) = crate::alu::daa(cpu.regs.a, cpu.regs.f);
            cpu.regs.a = r;
            cpu.regs.f = f;
            fetch!();
        }
        40 => {
            goto!(618);
        }
        41 => {
            crate::alu::add16(cpu, cpu.regs.hlx(cpu.state.hlx_idx));
            goto!(626);
        }
        42 => {
            goto!(633);
        }
        43 => {
            let v = cpu.regs.hlx(cpu.state.hlx_idx).wrapping_sub(1);
            cpu.regs.set_hlx(cpu.state.hlx_idx, v);
            goto!(645);
        }
        44 => {
            let v = inc8!(cpu.regs.hlx_l(cpu.state.hlx_idx));
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, v);
            fetch!();
        }
        45 => {
            let v = dec8!(cpu.regs.hlx_l(cpu.state.hlx_idx));
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, v);
            fetch!();
        }
        46 => {
            goto!(647);
        }
        47 => {
            let (r, f) = crate::alu::cpl(cpu.regs.a, cpu.regs.f);
            cpu.regs.a = r;
            cpu.regs.f = f;
            fetch!();
        }
        48 => {
            goto!(650);
        }
        49 => {
            goto!(658);
        }
        50 => {
            goto!(664);
        }
        51 => {
            cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
            goto!(673);
        }
        52 => {
            goto!(675);
        }
        53 => {
            goto!(682);
        }
        54 => {
            goto!(689);
        }
        55 => {
            cpu.regs.f = crate::alu::scf(cpu.regs.a, cpu.regs.f, cpu.revision);
            fetch!();
        }
        56 => {
            goto!(695);
        }
        57 => {
            crate::alu::add16(cpu, cpu.regs.sp);
            goto!(703);
        }
        58 => {
            goto!(710);
        }
        59 => {
            cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
            goto!(719);
        }
        60 => {
            cpu.regs.a = inc8!(cpu.regs.a);
            fetch!();
        }
        61 => {
            cpu.regs.a = dec8!(cpu.regs.a);
            fetch!();
        }
        62 => {
            goto!(721);
        }
        63 => {
            cpu.regs.f = crate::alu::ccf(cpu.regs.a, cpu.regs.f, cpu.revision);
            fetch!();
        }
        64 => {
            fetch!();
        }
        65 => {
            cpu.regs.b = cpu.regs.c;
            fetch!();
        }
        66 => {
            cpu.regs.b = cpu.regs.d;
            fetch!();
        }
        67 => {
            cpu.regs.b = cpu.regs.e;
            fetch!();
        }
        68 => {
            cpu.regs.b = cpu.regs.hlx_h(cpu.state.hlx_idx);
            fetch!();
        }
        69 => {
            cpu.regs.b = cpu.regs.hlx_l(cpu.state.hlx_idx);
            fetch!();
        }
        70 => {
            goto!(724);
        }
        71 => {
            cpu.regs.b = cpu.regs.a;
            fetch!();
        }
        72 => {
            cpu.regs.c = cpu.regs.b;
            fetch!();
        }
        73 => {
            fetch!();
        }
        74 => {
            cpu.regs.c = cpu.regs.d;
            fetch!();
        }
        75 => {
            cpu.regs.c = cpu.regs.e;
            fetch!();
        }
        76 => {
            cpu.regs.c = cpu.regs.hlx_h(cpu.state.hlx_idx);
            fetch!();
        }
        77 => {
            cpu.regs.c = cpu.regs.hlx_l(cpu.state.hlx_idx);
            fetch!();
        }
        78 => {
            goto!(727);
        }
        79 => {
            cpu.regs.c = cpu.regs.a;
            fetch!();
        }
        80 => {
            cpu.regs.d = cpu.regs.b;
            fetch!();
        }
        81 => {
            cpu.regs.d = cpu.regs.c;
            fetch!();
        }
        82 => {
            fetch!();
        }
        83 => {
            cpu.regs.d = cpu.regs.e;
            fetch!();
        }
        84 => {
            cpu.regs.d = cpu.regs.hlx_h(cpu.state.hlx_idx);
            fetch!();
        }
        85 => {
            cpu.regs.d = cpu.regs.hlx_l(cpu.state.hlx_idx);
            fetch!();
        }
        86 => {
            goto!(730);
        }
        87 => {
            cpu.regs.d = cpu.regs.a;
            fetch!();
        }
        88 => {
            cpu.regs.e = cpu.regs.b;
            fetch!();
        }
        89 => {
            cpu.regs.e = cpu.regs.c;
            fetch!();
        }
        90 => {
            cpu.regs.e = cpu.regs.d;
            fetch!();
        }
        91 => {
            fetch!();
        }
        92 => {
            cpu.regs.e = cpu.regs.hlx_h(cpu.state.hlx_idx);
            fetch!();
        }
        93 => {
            cpu.regs.e = cpu.regs.hlx_l(cpu.state.hlx_idx);
            fetch!();
        }
        94 => {
            goto!(733);
        }
        95 => {
            cpu.regs.e = cpu.regs.a;
            fetch!();
        }
        96 => {
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, cpu.regs.b);
            fetch!();
        }
        97 => {
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, cpu.regs.c);
            fetch!();
        }
        98 => {
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, cpu.regs.d);
            fetch!();
        }
        99 => {
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, cpu.regs.e);
            fetch!();
        }
        100 => {
            cpu.regs
                .set_hlx_h(cpu.state.hlx_idx, cpu.regs.hlx_h(cpu.state.hlx_idx));
            fetch!();
        }
        101 => {
            cpu.regs
                .set_hlx_h(cpu.state.hlx_idx, cpu.regs.hlx_l(cpu.state.hlx_idx));
            fetch!();
        }
        102 => {
            goto!(736);
        }
        103 => {
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, cpu.regs.a);
            fetch!();
        }
        104 => {
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, cpu.regs.b);
            fetch!();
        }
        105 => {
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, cpu.regs.c);
            fetch!();
        }
        106 => {
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, cpu.regs.d);
            fetch!();
        }
        107 => {
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, cpu.regs.e);
            fetch!();
        }
        108 => {
            cpu.regs
                .set_hlx_l(cpu.state.hlx_idx, cpu.regs.hlx_h(cpu.state.hlx_idx));
            fetch!();
        }
        109 => {
            cpu.regs
                .set_hlx_l(cpu.state.hlx_idx, cpu.regs.hlx_l(cpu.state.hlx_idx));
            fetch!();
        }
        110 => {
            goto!(739);
        }
        111 => {
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, cpu.regs.a);
            fetch!();
        }
        112 => {
            goto!(742);
        }
        113 => {
            goto!(745);
        }
        114 => {
            goto!(748);
        }
        115 => {
            goto!(751);
        }
        116 => {
            goto!(754);
        }
        117 => {
            goto!(757);
        }
        118 => {
            pins = cpu.halt(pins);
            fetch!();
        }
        119 => {
            goto!(760);
        }
        120 => {
            cpu.regs.a = cpu.regs.b;
            fetch!();
        }
        121 => {
            cpu.regs.a = cpu.regs.c;
            fetch!();
        }
        122 => {
            cpu.regs.a = cpu.regs.d;
            fetch!();
        }
        123 => {
            cpu.regs.a = cpu.regs.e;
            fetch!();
        }
        124 => {
            cpu.regs.a = cpu.regs.hlx_h(cpu.state.hlx_idx);
            fetch!();
        }
        125 => {
            cpu.regs.a = cpu.regs.hlx_l(cpu.state.hlx_idx);
            fetch!();
        }
        126 => {
            goto!(763);
        }
        127 => {
            fetch!();
        }
        128 => {
            add8!(cpu.regs.b);
            fetch!();
        }
        129 => {
            add8!(cpu.regs.c);
            fetch!();
        }
        130 => {
            add8!(cpu.regs.d);
            fetch!();
        }
        131 => {
            add8!(cpu.regs.e);
            fetch!();
        }
        132 => {
            add8!(cpu.regs.hlx_h(cpu.state.hlx_idx));
            fetch!();
        }
        133 => {
            add8!(cpu.regs.hlx_l(cpu.state.hlx_idx));
            fetch!();
        }
        134 => {
            goto!(766);
        }
        135 => {
            add8!(cpu.regs.a);
            fetch!();
        }
        136 => {
            adc8!(cpu.regs.b);
            fetch!();
        }
        137 => {
            adc8!(cpu.regs.c);
            fetch!();
        }
        138 => {
            adc8!(cpu.regs.d);
            fetch!();
        }
        139 => {
            adc8!(cpu.regs.e);
            fetch!();
        }
        140 => {
            adc8!(cpu.regs.hlx_h(cpu.state.hlx_idx));
            fetch!();
        }
        141 => {
            adc8!(cpu.regs.hlx_l(cpu.state.hlx_idx));
            fetch!();
        }
        142 => {
            goto!(769);
        }
        143 => {
            adc8!(cpu.regs.a);
            fetch!();
        }
        144 => {
            sub8!(cpu.regs.b);
            fetch!();
        }
        145 => {
            sub8!(cpu.regs.c);
            fetch!();
        }
        146 => {
            sub8!(cpu.regs.d);
            fetch!();
        }
        147 => {
            sub8!(cpu.regs.e);
            fetch!();
        }
        148 => {
            sub8!(cpu.regs.hlx_h(cpu.state.hlx_idx));
            fetch!();
        }
        149 => {
            sub8!(cpu.regs.hlx_l(cpu.state.hlx_idx));
            fetch!();
        }
        150 => {
            goto!(772);
        }
        151 => {
            sub8!(cpu.regs.a);
            fetch!();
        }
        152 => {
            sbc8!(cpu.regs.b);
            fetch!();
        }
        153 => {
            sbc8!(cpu.regs.c);
            fetch!();
        }
        154 => {
            sbc8!(cpu.regs.d);
            fetch!();
        }
        155 => {
            sbc8!(cpu.regs.e);
            fetch!();
        }
        156 => {
            sbc8!(cpu.regs.hlx_h(cpu.state.hlx_idx));
            fetch!();
        }
        157 => {
            sbc8!(cpu.regs.hlx_l(cpu.state.hlx_idx));
            fetch!();
        }
        158 => {
            goto!(775);
        }
        159 => {
            sbc8!(cpu.regs.a);
            fetch!();
        }
        160 => {
            and8!(cpu.regs.b);
            fetch!();
        }
        161 => {
            and8!(cpu.regs.c);
            fetch!();
        }
        162 => {
            and8!(cpu.regs.d);
            fetch!();
        }
        163 => {
            and8!(cpu.regs.e);
            fetch!();
        }
        164 => {
            and8!(cpu.regs.hlx_h(cpu.state.hlx_idx));
            fetch!();
        }
        165 => {
            and8!(cpu.regs.hlx_l(cpu.state.hlx_idx));
            fetch!();
        }
        166 => {
            goto!(778);
        }
        167 => {
            and8!(cpu.regs.a);
            fetch!();
        }
        168 => {
            xor8!(cpu.regs.b);
            fetch!();
        }
        169 => {
            xor8!(cpu.regs.c);
            fetch!();
        }
        170 => {
            xor8!(cpu.regs.d);
            fetch!();
        }
        171 => {
            xor8!(cpu.regs.e);
            fetch!();
        }
        172 => {
            xor8!(cpu.regs.hlx_h(cpu.state.hlx_idx));
            fetch!();
        }
        173 => {
            xor8!(cpu.regs.hlx_l(cpu.state.hlx_idx));
            fetch!();
        }
        174 => {
            goto!(781);
        }
        175 => {
            xor8!(cpu.regs.a);
            fetch!();
        }
        176 => {
            or8!(cpu.regs.b);
            fetch!();
        }
        177 => {
            or8!(cpu.regs.c);
            fetch!();
        }
        178 => {
            or8!(cpu.regs.d);
            fetch!();
        }
        179 => {
            or8!(cpu.regs.e);
            fetch!();
        }
        180 => {
            or8!(cpu.regs.hlx_h(cpu.state.hlx_idx));
            fetch!();
        }
        181 => {
            or8!(cpu.regs.hlx_l(cpu.state.hlx_idx));
            fetch!();
        }
        182 => {
            goto!(784);
        }
        183 => {
            or8!(cpu.regs.a);
            fetch!();
        }
        184 => {
            cp8!(cpu.regs.b);
            fetch!();
        }
        185 => {
            cp8!(cpu.regs.c);
            fetch!();
        }
        186 => {
            cp8!(cpu.regs.d);
            fetch!();
        }
        187 => {
            cp8!(cpu.regs.e);
            fetch!();
        }
        188 => {
            cp8!(cpu.regs.hlx_h(cpu.state.hlx_idx));
            fetch!();
        }
        189 => {
            cp8!(cpu.regs.hlx_l(cpu.state.hlx_idx));
            fetch!();
        }
        190 => {
            goto!(787);
        }
        191 => {
            cp8!(cpu.regs.a);
            fetch!();
        }
        192 => {
            if !cc_nz!() {
                goto!(796);
            }
            goto!(790);
        }
        193 => {
            goto!(797);
        }
        194 => {
            goto!(803);
        }
        195 => {
            goto!(809);
        }
        196 => {
            goto!(815);
        }
        197 => {
            goto!(828);
        }
        198 => {
            goto!(835);
        }
        199 => {
            goto!(838);
        }
        200 => {
            if !cc_z!() {
                goto!(851);
            }
            goto!(845);
        }
        201 => {
            goto!(852);
        }
        202 => {
            goto!(858);
        }
        203 => {
            fetch_cb!();
        }
        204 => {
            goto!(864);
        }
        205 => {
            goto!(877);
        }
        206 => {
            goto!(890);
        }
        207 => {
            goto!(893);
        }
        208 => {
            if !cc_nc!() {
                goto!(906);
            }
            goto!(900);
        }
        209 => {
            goto!(907);
        }
        210 => {
            goto!(913);
        }
        211 => {
            goto!(919);
        }
        212 => {
            goto!(926);
        }
        213 => {
            goto!(939);
        }
        214 => {
            goto!(946);
        }
        215 => {
            goto!(949);
        }
        216 => {
            if !cc_c!() {
                goto!(962);
            }
            goto!(956);
        }
        217 => {
            cpu.regs.swap_exx();
            fetch!();
        }
        218 => {
            goto!(963);
        }
        219 => {
            goto!(969);
        }
        220 => {
            goto!(976);
        }
        221 => {
            fetch_dd!();
        }
        222 => {
            goto!(989);
        }
        223 => {
            goto!(992);
        }
        224 => {
            if !cc_po!() {
                goto!(1005);
            }
            goto!(999);
        }
        225 => {
            goto!(1006);
        }
        226 => {
            goto!(1012);
        }
        227 => {
            goto!(1018);
        }
        228 => {
            goto!(1033);
        }
        229 => {
            goto!(1046);
        }
        230 => {
            goto!(1053);
        }
        231 => {
            goto!(1056);
        }
        232 => {
            if !cc_pe!() {
                goto!(1069);
            }
            goto!(1063);
        }
        233 => {
            cpu.regs.pc = cpu.regs.hlx(cpu.state.hlx_idx);
            fetch!();
        }
        234 => {
            goto!(1070);
        }
        235 => {
            let tmp = cpu.regs.hl();
            cpu.regs.set_hl(cpu.regs.de());
            cpu.regs.set_de(tmp);
            fetch!();
        }
        236 => {
            goto!(1076);
        }
        237 => {
            fetch_ed!();
        }
        238 => {
            goto!(1089);
        }
        239 => {
            goto!(1092);
        }
        240 => {
            if !cc_p!() {
                goto!(1105);
            }
            goto!(1099);
        }
        241 => {
            goto!(1106);
        }
        242 => {
            goto!(1112);
        }
        243 => {
            cpu.state.iff1 = false;
            cpu.state.iff2 = false;
            fetch!();
        }
        244 => {
            goto!(1118);
        }
        245 => {
            goto!(1131);
        }
        246 => {
            goto!(1138);
        }
        247 => {
            goto!(1141);
        }
        248 => {
            if !cc_m!() {
                goto!(1154);
            }
            goto!(1148);
        }
        249 => {
            cpu.regs.sp = cpu.regs.hlx(cpu.state.hlx_idx);
            goto!(1155);
        }
        250 => {
            goto!(1157);
        }
        251 => {
            cpu.state.iff1 = false;
            cpu.state.iff2 = false;
            pins = cpu.fetch(pins);
            cpu.state.iff1 = true;
            cpu.state.iff2 = true;
            pins
        }
        252 => {
            goto!(1163);
        }
        253 => {
            fetch_fd!();
        }
        254 => {
            goto!(1176);
        }
        255 => {
            goto!(1179);
        }

        256..=319 | 375 | 383..=415 | 420..=423 | 428..=431 | 436..=439 | 444..=511 => {
            fetch!();
        }

        320 => {
            goto!(1186);
        }
        321 => {
            goto!(1190);
        }
        322 => {
            crate::alu::sbc16(cpu, cpu.regs.bc());
            goto!(1194);
        }
        323 => {
            goto!(1201);
        }
        324 => {
            neg8!();
            fetch!();
        }
        325 => {
            goto!(1213);
        }
        326 => {
            cpu.state.im = crate::state::InterruptMode::IM0;
            fetch!();
        }
        327 => {
            goto!(1219);
        }
        328 => {
            goto!(1220);
        }
        329 => {
            goto!(1224);
        }
        330 => {
            crate::alu::adc16(cpu, cpu.regs.bc());
            goto!(1228);
        }
        331 => {
            goto!(1235);
        }
        332 => {
            neg8!();
            fetch!();
        }
        333 => {
            goto!(1247);
        }
        334 => {
            cpu.state.im = crate::state::InterruptMode::IM0;
            fetch!();
        }
        335 => {
            goto!(1253);
        }
        336 => {
            goto!(1254);
        }
        337 => {
            goto!(1258);
        }
        338 => {
            crate::alu::sbc16(cpu, cpu.regs.de());
            goto!(1262);
        }
        339 => {
            goto!(1269);
        }
        340 => {
            neg8!();
            fetch!();
        }
        341 => {
            goto!(1247);
        }
        342 => {
            cpu.state.im = crate::state::InterruptMode::IM1;
            fetch!();
        }
        343 => {
            goto!(1282);
        }
        344 => {
            goto!(1283);
        }
        345 => {
            goto!(1287);
        }
        346 => {
            crate::alu::adc16(cpu, cpu.regs.de());
            goto!(1291);
        }
        347 => {
            goto!(1298);
        }
        348 => {
            neg8!();
            fetch!();
        }
        349 => {
            goto!(1247);
        }
        350 => {
            cpu.state.im = crate::state::InterruptMode::IM2;
            fetch!();
        }
        351 => {
            goto!(1311);
        }
        352 => {
            goto!(1312);
        }
        353 => {
            goto!(1316);
        }
        354 => {
            crate::alu::sbc16(cpu, cpu.regs.hl());
            goto!(1320);
        }
        355 => {
            goto!(1327);
        }
        356 => {
            neg8!();
            fetch!();
        }
        357 => {
            goto!(1247);
        }
        358 => {
            cpu.state.im = crate::state::InterruptMode::IM0;
            fetch!();
        }
        359 => {
            goto!(1340);
        }
        360 => {
            goto!(1350);
        }
        361 => {
            goto!(1354);
        }
        362 => {
            crate::alu::adc16(cpu, cpu.regs.hl());
            goto!(1358);
        }
        363 => {
            goto!(1365);
        }
        364 => {
            neg8!();
            fetch!();
        }
        365 => {
            goto!(1247);
        }
        366 => {
            cpu.state.im = crate::state::InterruptMode::IM0;
            fetch!();
        }
        367 => {
            goto!(1378);
        }
        368 => {
            goto!(1388);
        }
        369 => {
            goto!(1392);
        }
        370 => {
            crate::alu::sbc16(cpu, cpu.regs.sp);
            goto!(1396);
        }
        371 => {
            goto!(1403);
        }
        372 => {
            neg8!();
            fetch!();
        }
        373 => {
            goto!(1247);
        }
        374 => {
            cpu.state.im = crate::state::InterruptMode::IM1;
            fetch!();
        }
        376 => {
            goto!(1416);
        }
        377 => {
            goto!(1420);
        }
        378 => {
            crate::alu::adc16(cpu, cpu.regs.sp);
            goto!(1424);
        }
        379 => {
            goto!(1431);
        }
        380 => {
            neg8!();
            fetch!();
        }
        381 => {
            goto!(1247);
        }
        382 => {
            cpu.state.im = crate::state::InterruptMode::IM2;
            fetch!();
        }
        416 => {
            goto!(1444);
        }
        417 => {
            goto!(1452);
        }
        418 => {
            goto!(1460);
        }
        419 => {
            goto!(1468);
        }
        424 => {
            goto!(1476);
        }
        425 => {
            goto!(1484);
        }
        426 => {
            goto!(1492);
        }
        427 => {
            goto!(1500);
        }
        432 => {
            goto!(1508);
        }
        433 => {
            goto!(1521);
        }
        434 => {
            goto!(1534);
        }
        435 => {
            goto!(1547);
        }
        440 => {
            goto!(1560);
        }
        441 => {
            goto!(1573);
        }
        442 => {
            goto!(1586);
        }
        443 => {
            goto!(1599);
        }

        512 => {
            wait!();
            mread!(pc_postinc!());
            goto!(513);
        }
        513 => {
            cpu.regs.c = gd!();
            goto!(514);
        }
        514 => {
            goto!(515);
        }
        515 => {
            wait!();
            mread!(pc_postinc!());
            goto!(516);
        }
        516 => {
            cpu.regs.b = gd!();
            goto!(517);
        }
        517 => {
            fetch!();
        }
        518 => {
            wait!();
            mwrite!(cpu.regs.bc(), cpu.regs.a);
            cpu.regs.set_wz_l(cpu.regs.c.wrapping_add(1));
            cpu.regs.set_wz_h(cpu.regs.a);
            goto!(519);
        }
        519 => {
            goto!(520);
        }
        520 => {
            fetch!();
        }
        521 => {
            goto!(522);
        }
        522 => {
            fetch!();
        }
        523 => {
            wait!();
            mread!(pc_postinc!());
            goto!(524);
        }
        524 => {
            cpu.regs.b = gd!();
            goto!(525);
        }
        525 => {
            fetch!();
        }
        526 => {
            goto!(527);
        }
        527 => {
            goto!(528);
        }
        528 => {
            goto!(529);
        }
        529 => {
            goto!(530);
        }
        530 => {
            goto!(531);
        }
        531 => {
            goto!(532);
        }
        532 => {
            fetch!();
        }
        533 => {
            wait!();
            mread!(cpu.regs.bc());
            goto!(534);
        }
        534 => {
            cpu.regs.a = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(535);
        }
        535 => {
            fetch!();
        }
        536 => {
            goto!(537);
        }
        537 => {
            fetch!();
        }
        538 => {
            wait!();
            mread!(pc_postinc!());
            goto!(539);
        }
        539 => {
            cpu.regs.c = gd!();
            goto!(540);
        }
        540 => {
            fetch!();
        }
        541 => {
            goto!(542);
        }
        542 => {
            wait!();
            mread!(pc_postinc!());
            goto!(543);
        }
        543 => {
            cpu.state.dlatch = gd!();
            cpu.regs.b = cpu.regs.b.wrapping_sub(1);
            if cpu.regs.b == 0 {
                goto!(549);
            }
            goto!(544);
        }
        544 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_add(cpu.state.dlatch as i8 as u16);
            cpu.regs.wz = cpu.regs.pc;
            goto!(545);
        }
        545 => {
            goto!(546);
        }
        546 => {
            goto!(547);
        }
        547 => {
            goto!(548);
        }
        548 => {
            goto!(549);
        }
        549 => {
            fetch!();
        }
        550 => {
            wait!();
            mread!(pc_postinc!());
            goto!(551);
        }
        551 => {
            cpu.regs.e = gd!();
            goto!(552);
        }
        552 => {
            goto!(553);
        }
        553 => {
            wait!();
            mread!(pc_postinc!());
            goto!(554);
        }
        554 => {
            cpu.regs.d = gd!();
            goto!(555);
        }
        555 => {
            fetch!();
        }
        556 => {
            wait!();
            mwrite!(cpu.regs.de(), cpu.regs.a);
            cpu.regs.set_wz_l(cpu.regs.e.wrapping_add(1));
            cpu.regs.set_wz_h(cpu.regs.a);
            goto!(557);
        }
        557 => {
            goto!(558);
        }
        558 => {
            fetch!();
        }
        559 => {
            goto!(560);
        }
        560 => {
            fetch!();
        }
        561 => {
            wait!();
            mread!(pc_postinc!());
            goto!(562);
        }
        562 => {
            cpu.regs.d = gd!();
            goto!(563);
        }
        563 => {
            fetch!();
        }
        564 => {
            wait!();
            mread!(pc_postinc!());
            goto!(565);
        }
        565 => {
            cpu.state.dlatch = gd!();
            goto!(566);
        }
        566 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_add(cpu.state.dlatch as i8 as u16);
            cpu.regs.wz = cpu.regs.pc;
            goto!(567);
        }
        567 => {
            goto!(568);
        }
        568 => {
            goto!(569);
        }
        569 => {
            goto!(570);
        }
        570 => {
            goto!(571);
        }
        571 => {
            fetch!();
        }
        572 => {
            goto!(573);
        }
        573 => {
            goto!(574);
        }
        574 => {
            goto!(575);
        }
        575 => {
            goto!(576);
        }
        576 => {
            goto!(577);
        }
        577 => {
            goto!(578);
        }
        578 => {
            fetch!();
        }
        579 => {
            wait!();
            mread!(cpu.regs.de());
            goto!(580);
        }
        580 => {
            cpu.regs.a = gd!();
            cpu.regs.wz = cpu.regs.de().wrapping_add(1);
            goto!(581);
        }
        581 => {
            fetch!();
        }
        582 => {
            goto!(583);
        }
        583 => {
            fetch!();
        }
        584 => {
            wait!();
            mread!(pc_postinc!());
            goto!(585);
        }
        585 => {
            cpu.regs.e = gd!();
            goto!(586);
        }
        586 => {
            fetch!();
        }
        587 => {
            wait!();
            mread!(pc_postinc!());
            goto!(588);
        }
        588 => {
            cpu.state.dlatch = gd!();
            if !cc_nz!() {
                goto!(594);
            }
            goto!(589);
        }
        589 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_add(cpu.state.dlatch as i8 as u16);
            cpu.regs.wz = cpu.regs.pc;
            goto!(590);
        }
        590 => {
            goto!(591);
        }
        591 => {
            goto!(592);
        }
        592 => {
            goto!(593);
        }
        593 => {
            goto!(594);
        }
        594 => {
            fetch!();
        }
        595 => {
            wait!();
            mread!(pc_postinc!());
            goto!(596);
        }
        596 => {
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, gd!());
            goto!(597);
        }
        597 => {
            goto!(598);
        }
        598 => {
            wait!();
            mread!(pc_postinc!());
            goto!(599);
        }
        599 => {
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, gd!());
            goto!(600);
        }
        600 => {
            fetch!();
        }
        601 => {
            wait!();
            mread!(pc_postinc!());
            goto!(602);
        }
        602 => {
            cpu.regs.set_wz_l(gd!());
            goto!(603);
        }
        603 => {
            goto!(604);
        }
        604 => {
            wait!();
            mread!(pc_postinc!());
            goto!(605);
        }
        605 => {
            cpu.regs.set_wz_h(gd!());
            goto!(606);
        }
        606 => {
            goto!(607);
        }
        607 => {
            wait!();
            mwrite!(wz_postinc!(), cpu.regs.hlx_l(cpu.state.hlx_idx));
            goto!(608);
        }
        608 => {
            goto!(609);
        }
        609 => {
            goto!(610);
        }
        610 => {
            wait!();
            mwrite!(cpu.regs.wz, cpu.regs.hlx_h(cpu.state.hlx_idx));
            goto!(611);
        }
        611 => {
            goto!(612);
        }
        612 => {
            fetch!();
        }
        613 => {
            goto!(614);
        }
        614 => {
            fetch!();
        }
        615 => {
            wait!();
            mread!(pc_postinc!());
            goto!(616);
        }
        616 => {
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, gd!());
            goto!(617);
        }
        617 => {
            fetch!();
        }
        618 => {
            wait!();
            mread!(pc_postinc!());
            goto!(619);
        }
        619 => {
            cpu.state.dlatch = gd!();
            if !cc_z!() {
                goto!(625);
            }
            goto!(620);
        }
        620 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_add(cpu.state.dlatch as i8 as u16);
            cpu.regs.wz = cpu.regs.pc;
            goto!(621);
        }
        621 => {
            goto!(622);
        }
        622 => {
            goto!(623);
        }
        623 => {
            goto!(624);
        }
        624 => {
            goto!(625);
        }
        625 => {
            fetch!();
        }
        626 => {
            goto!(627);
        }
        627 => {
            goto!(628);
        }
        628 => {
            goto!(629);
        }
        629 => {
            goto!(630);
        }
        630 => {
            goto!(631);
        }
        631 => {
            goto!(632);
        }
        632 => {
            fetch!();
        }
        633 => {
            wait!();
            mread!(pc_postinc!());
            goto!(634);
        }
        634 => {
            cpu.regs.set_wz_l(gd!());
            goto!(635);
        }
        635 => {
            goto!(636);
        }
        636 => {
            wait!();
            mread!(pc_postinc!());
            goto!(637);
        }
        637 => {
            cpu.regs.set_wz_h(gd!());
            goto!(638);
        }
        638 => {
            goto!(639);
        }
        639 => {
            wait!();
            mread!(wz_postinc!());
            goto!(640);
        }
        640 => {
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, gd!());
            goto!(641);
        }
        641 => {
            goto!(642);
        }
        642 => {
            wait!();
            mread!(cpu.regs.wz);
            goto!(643);
        }
        643 => {
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, gd!());
            goto!(644);
        }
        644 => {
            fetch!();
        }
        645 => {
            goto!(646);
        }
        646 => {
            fetch!();
        }
        647 => {
            wait!();
            mread!(pc_postinc!());
            goto!(648);
        }
        648 => {
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, gd!());
            goto!(649);
        }
        649 => {
            fetch!();
        }
        650 => {
            wait!();
            mread!(pc_postinc!());
            goto!(651);
        }
        651 => {
            cpu.state.dlatch = gd!();
            if !cc_nc!() {
                goto!(657);
            }
            goto!(652);
        }
        652 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_add(cpu.state.dlatch as i8 as u16);
            cpu.regs.wz = cpu.regs.pc;
            goto!(653);
        }
        653 => {
            goto!(654);
        }
        654 => {
            goto!(655);
        }
        655 => {
            goto!(656);
        }
        656 => {
            goto!(657);
        }
        657 => {
            fetch!();
        }
        658 => {
            wait!();
            mread!(pc_postinc!());
            goto!(659);
        }
        659 => {
            cpu.regs.set_sp_l(gd!());
            goto!(660);
        }
        660 => {
            goto!(661);
        }
        661 => {
            wait!();
            mread!(pc_postinc!());
            goto!(662);
        }
        662 => {
            cpu.regs.set_sp_h(gd!());
            goto!(663);
        }
        663 => {
            fetch!();
        }
        664 => {
            wait!();
            mread!(pc_postinc!());
            goto!(665);
        }
        665 => {
            cpu.regs.set_wz_l(gd!());
            goto!(666);
        }
        666 => {
            goto!(667);
        }
        667 => {
            wait!();
            mread!(pc_postinc!());
            goto!(668);
        }
        668 => {
            cpu.regs.set_wz_h(gd!());
            goto!(669);
        }
        669 => {
            goto!(670);
        }
        670 => {
            wait!();
            mwrite!(wz_postinc!(), cpu.regs.a);
            cpu.regs.set_wz_h(cpu.regs.a);
            goto!(671);
        }
        671 => {
            goto!(672);
        }
        672 => {
            fetch!();
        }
        673 => {
            goto!(674);
        }
        674 => {
            fetch!();
        }
        675 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(676);
        }
        676 => {
            cpu.state.dlatch = gd!();
            cpu.state.dlatch = inc8!(cpu.state.dlatch);
            goto!(677);
        }
        677 => {
            goto!(678);
        }
        678 => {
            goto!(679);
        }
        679 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.state.dlatch);
            goto!(680);
        }
        680 => {
            goto!(681);
        }
        681 => {
            fetch!();
        }
        682 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(683);
        }
        683 => {
            cpu.state.dlatch = gd!();
            cpu.state.dlatch = dec8!(cpu.state.dlatch);
            goto!(684);
        }
        684 => {
            goto!(685);
        }
        685 => {
            goto!(686);
        }
        686 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.state.dlatch);
            goto!(687);
        }
        687 => {
            goto!(688);
        }
        688 => {
            fetch!();
        }
        689 => {
            wait!();
            mread!(pc_postinc!());
            goto!(690);
        }
        690 => {
            cpu.state.dlatch = gd!();
            goto!(691);
        }
        691 => {
            goto!(692);
        }
        692 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.state.dlatch);
            goto!(693);
        }
        693 => {
            goto!(694);
        }
        694 => {
            fetch!();
        }
        695 => {
            wait!();
            mread!(pc_postinc!());
            goto!(696);
        }
        696 => {
            cpu.state.dlatch = gd!();
            if !cc_c!() {
                goto!(702);
            }
            goto!(697);
        }
        697 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_add(cpu.state.dlatch as i8 as u16);
            cpu.regs.wz = cpu.regs.pc;
            goto!(698);
        }
        698 => {
            goto!(699);
        }
        699 => {
            goto!(700);
        }
        700 => {
            goto!(701);
        }
        701 => {
            goto!(702);
        }
        702 => {
            fetch!();
        }
        703 => {
            goto!(704);
        }
        704 => {
            goto!(705);
        }
        705 => {
            goto!(706);
        }
        706 => {
            goto!(707);
        }
        707 => {
            goto!(708);
        }
        708 => {
            goto!(709);
        }
        709 => {
            fetch!();
        }
        710 => {
            wait!();
            mread!(pc_postinc!());
            goto!(711);
        }
        711 => {
            cpu.regs.set_wz_l(gd!());
            goto!(712);
        }
        712 => {
            goto!(713);
        }
        713 => {
            wait!();
            mread!(pc_postinc!());
            goto!(714);
        }
        714 => {
            cpu.regs.set_wz_h(gd!());
            goto!(715);
        }
        715 => {
            goto!(716);
        }
        716 => {
            wait!();
            mread!(wz_postinc!());
            goto!(717);
        }
        717 => {
            cpu.regs.a = gd!();
            goto!(718);
        }
        718 => {
            fetch!();
        }
        719 => {
            goto!(720);
        }
        720 => {
            fetch!();
        }
        721 => {
            wait!();
            mread!(pc_postinc!());
            goto!(722);
        }
        722 => {
            cpu.regs.a = gd!();
            goto!(723);
        }
        723 => {
            fetch!();
        }
        724 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(725);
        }
        725 => {
            cpu.regs.b = gd!();
            goto!(726);
        }
        726 => {
            fetch!();
        }
        727 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(728);
        }
        728 => {
            cpu.regs.c = gd!();
            goto!(729);
        }
        729 => {
            fetch!();
        }
        730 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(731);
        }
        731 => {
            cpu.regs.d = gd!();
            goto!(732);
        }
        732 => {
            fetch!();
        }
        733 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(734);
        }
        734 => {
            cpu.regs.e = gd!();
            goto!(735);
        }
        735 => {
            fetch!();
        }
        736 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(737);
        }
        737 => {
            cpu.regs.h = gd!();
            goto!(738);
        }
        738 => {
            fetch!();
        }
        739 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(740);
        }
        740 => {
            cpu.regs.l = gd!();
            goto!(741);
        }
        741 => {
            fetch!();
        }
        742 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.regs.b);
            goto!(743);
        }
        743 => {
            goto!(744);
        }
        744 => {
            fetch!();
        }
        745 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.regs.c);
            goto!(746);
        }
        746 => {
            goto!(747);
        }
        747 => {
            fetch!();
        }
        748 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.regs.d);
            goto!(749);
        }
        749 => {
            goto!(750);
        }
        750 => {
            fetch!();
        }
        751 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.regs.e);
            goto!(752);
        }
        752 => {
            goto!(753);
        }
        753 => {
            fetch!();
        }
        754 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.regs.h);
            goto!(755);
        }
        755 => {
            goto!(756);
        }
        756 => {
            fetch!();
        }
        757 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.regs.l);
            goto!(758);
        }
        758 => {
            goto!(759);
        }
        759 => {
            fetch!();
        }
        760 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.regs.a);
            goto!(761);
        }
        761 => {
            goto!(762);
        }
        762 => {
            fetch!();
        }
        763 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(764);
        }
        764 => {
            cpu.regs.a = gd!();
            goto!(765);
        }
        765 => {
            fetch!();
        }
        766 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(767);
        }
        767 => {
            cpu.state.dlatch = gd!();
            goto!(768);
        }
        768 => {
            add8!(cpu.state.dlatch);
            fetch!();
        }
        769 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(770);
        }
        770 => {
            cpu.state.dlatch = gd!();
            goto!(771);
        }
        771 => {
            adc8!(cpu.state.dlatch);
            fetch!();
        }
        772 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(773);
        }
        773 => {
            cpu.state.dlatch = gd!();
            goto!(774);
        }
        774 => {
            sub8!(cpu.state.dlatch);
            fetch!();
        }
        775 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(776);
        }
        776 => {
            cpu.state.dlatch = gd!();
            goto!(777);
        }
        777 => {
            sbc8!(cpu.state.dlatch);
            fetch!();
        }
        778 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(779);
        }
        779 => {
            cpu.state.dlatch = gd!();
            goto!(780);
        }
        780 => {
            and8!(cpu.state.dlatch);
            fetch!();
        }
        781 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(782);
        }
        782 => {
            cpu.state.dlatch = gd!();
            goto!(783);
        }
        783 => {
            xor8!(cpu.state.dlatch);
            fetch!();
        }
        784 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(785);
        }
        785 => {
            cpu.state.dlatch = gd!();
            goto!(786);
        }
        786 => {
            or8!(cpu.state.dlatch);
            fetch!();
        }
        787 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(788);
        }
        788 => {
            cpu.state.dlatch = gd!();
            goto!(789);
        }
        789 => {
            cp8!(cpu.state.dlatch);
            fetch!();
        }
        790 => {
            goto!(791);
        }
        791 => {
            wait!();
            mread!(sp_postinc!());
            goto!(792);
        }
        792 => {
            cpu.regs.set_wz_l(gd!());
            goto!(793);
        }
        793 => {
            goto!(794);
        }
        794 => {
            wait!();
            mread!(sp_postinc!());
            goto!(795);
        }
        795 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(796);
        }
        796 => {
            fetch!();
        }
        797 => {
            wait!();
            mread!(sp_postinc!());
            goto!(798);
        }
        798 => {
            cpu.regs.c = gd!();
            goto!(799);
        }
        799 => {
            goto!(800);
        }
        800 => {
            wait!();
            mread!(sp_postinc!());
            goto!(801);
        }
        801 => {
            cpu.regs.b = gd!();
            goto!(802);
        }
        802 => {
            fetch!();
        }
        803 => {
            wait!();
            mread!(pc_postinc!());
            goto!(804);
        }
        804 => {
            cpu.regs.set_wz_l(gd!());
            goto!(805);
        }
        805 => {
            goto!(806);
        }
        806 => {
            wait!();
            mread!(pc_postinc!());
            goto!(807);
        }
        807 => {
            cpu.regs.set_wz_h(gd!());
            if cc_nz!() {
                cpu.regs.pc = cpu.regs.wz;
            }
            goto!(808);
        }
        808 => {
            fetch!();
        }
        809 => {
            wait!();
            mread!(pc_postinc!());
            goto!(810);
        }
        810 => {
            cpu.regs.set_wz_l(gd!());
            goto!(811);
        }
        811 => {
            goto!(812);
        }
        812 => {
            wait!();
            mread!(pc_postinc!());
            goto!(813);
        }
        813 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(814);
        }
        814 => {
            fetch!();
        }
        815 => {
            wait!();
            mread!(pc_postinc!());
            goto!(816);
        }
        816 => {
            cpu.regs.set_wz_l(gd!());
            goto!(817);
        }
        817 => {
            goto!(818);
        }
        818 => {
            wait!();
            mread!(pc_postinc!());
            goto!(819);
        }
        819 => {
            cpu.regs.set_wz_h(gd!());
            if !cc_nz!() {
                goto!(827);
            }
            goto!(820);
        }
        820 => {
            goto!(821);
        }
        821 => {
            goto!(822);
        }
        822 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(823);
        }
        823 => {
            goto!(824);
        }
        824 => {
            goto!(825);
        }
        825 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.pc = cpu.regs.wz;
            goto!(826);
        }
        826 => {
            goto!(827);
        }
        827 => {
            fetch!();
        }
        828 => {
            goto!(829);
        }
        829 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.b);
            goto!(830);
        }
        830 => {
            goto!(831);
        }
        831 => {
            goto!(832);
        }
        832 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.c);
            goto!(833);
        }
        833 => {
            goto!(834);
        }
        834 => {
            fetch!();
        }
        835 => {
            wait!();
            mread!(pc_postinc!());
            goto!(836);
        }
        836 => {
            cpu.state.dlatch = gd!();
            goto!(837);
        }
        837 => {
            add8!(cpu.state.dlatch);
            fetch!();
        }
        838 => {
            goto!(839);
        }
        839 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(840);
        }
        840 => {
            goto!(841);
        }
        841 => {
            goto!(842);
        }
        842 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.wz = 0x00;
            cpu.regs.pc = cpu.regs.wz;
            goto!(843);
        }
        843 => {
            goto!(844);
        }
        844 => {
            fetch!();
        }
        845 => {
            goto!(846);
        }
        846 => {
            wait!();
            mread!(sp_postinc!());
            goto!(847);
        }
        847 => {
            cpu.regs.set_wz_l(gd!());
            goto!(848);
        }
        848 => {
            goto!(849);
        }
        849 => {
            wait!();
            mread!(sp_postinc!());
            goto!(850);
        }
        850 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(851);
        }
        851 => {
            fetch!();
        }
        852 => {
            wait!();
            mread!(sp_postinc!());
            goto!(853);
        }
        853 => {
            cpu.regs.set_wz_l(gd!());
            goto!(854);
        }
        854 => {
            goto!(855);
        }
        855 => {
            wait!();
            mread!(sp_postinc!());
            goto!(856);
        }
        856 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(857);
        }
        857 => {
            fetch!();
        }
        858 => {
            wait!();
            mread!(pc_postinc!());
            goto!(859);
        }
        859 => {
            cpu.regs.set_wz_l(gd!());
            goto!(860);
        }
        860 => {
            goto!(861);
        }
        861 => {
            wait!();
            mread!(pc_postinc!());
            goto!(862);
        }
        862 => {
            cpu.regs.set_wz_h(gd!());
            if cc_z!() {
                cpu.regs.pc = cpu.regs.wz;
            }
            goto!(863);
        }
        863 => {
            fetch!();
        }
        864 => {
            wait!();
            mread!(pc_postinc!());
            goto!(865);
        }
        865 => {
            cpu.regs.set_wz_l(gd!());
            goto!(866);
        }
        866 => {
            goto!(867);
        }
        867 => {
            wait!();
            mread!(pc_postinc!());
            goto!(868);
        }
        868 => {
            cpu.regs.set_wz_h(gd!());
            if !cc_z!() {
                goto!(876);
            }
            goto!(869);
        }
        869 => {
            goto!(870);
        }
        870 => {
            goto!(871);
        }
        871 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(872);
        }
        872 => {
            goto!(873);
        }
        873 => {
            goto!(874);
        }
        874 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.pc = cpu.regs.wz;
            goto!(875);
        }
        875 => {
            goto!(876);
        }
        876 => {
            fetch!();
        }
        877 => {
            wait!();
            mread!(pc_postinc!());
            goto!(878);
        }
        878 => {
            cpu.regs.set_wz_l(gd!());
            goto!(879);
        }
        879 => {
            goto!(880);
        }
        880 => {
            wait!();
            mread!(pc_postinc!());
            goto!(881);
        }
        881 => {
            cpu.regs.set_wz_h(gd!());
            goto!(882);
        }
        882 => {
            goto!(883);
        }
        883 => {
            goto!(884);
        }
        884 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(885);
        }
        885 => {
            goto!(886);
        }
        886 => {
            goto!(887);
        }
        887 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.pc = cpu.regs.wz;
            goto!(888);
        }
        888 => {
            goto!(889);
        }
        889 => {
            fetch!();
        }
        890 => {
            wait!();
            mread!(pc_postinc!());
            goto!(891);
        }
        891 => {
            cpu.state.dlatch = gd!();
            goto!(892);
        }
        892 => {
            adc8!(cpu.state.dlatch);
            fetch!();
        }
        893 => {
            goto!(894);
        }
        894 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(895);
        }
        895 => {
            goto!(896);
        }
        896 => {
            goto!(897);
        }
        897 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.wz = 0x08;
            cpu.regs.pc = cpu.regs.wz;
            goto!(898);
        }
        898 => {
            goto!(899);
        }
        899 => {
            fetch!();
        }
        900 => {
            goto!(901);
        }
        901 => {
            wait!();
            mread!(sp_postinc!());
            goto!(902);
        }
        902 => {
            cpu.regs.set_wz_l(gd!());
            goto!(903);
        }
        903 => {
            goto!(904);
        }
        904 => {
            wait!();
            mread!(sp_postinc!());
            goto!(905);
        }
        905 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(906);
        }
        906 => {
            fetch!();
        }
        907 => {
            wait!();
            mread!(sp_postinc!());
            goto!(908);
        }
        908 => {
            cpu.regs.e = gd!();
            goto!(909);
        }
        909 => {
            goto!(910);
        }
        910 => {
            wait!();
            mread!(sp_postinc!());
            goto!(911);
        }
        911 => {
            cpu.regs.d = gd!();
            goto!(912);
        }
        912 => {
            fetch!();
        }
        913 => {
            wait!();
            mread!(pc_postinc!());
            goto!(914);
        }
        914 => {
            cpu.regs.set_wz_l(gd!());
            goto!(915);
        }
        915 => {
            goto!(916);
        }
        916 => {
            wait!();
            mread!(pc_postinc!());
            goto!(917);
        }
        917 => {
            cpu.regs.set_wz_h(gd!());
            if cc_nc!() {
                cpu.regs.pc = cpu.regs.wz;
            }
            goto!(918);
        }
        918 => {
            fetch!();
        }
        919 => {
            wait!();
            mread!(pc_postinc!());
            goto!(920);
        }
        920 => {
            cpu.regs.set_wz_l(gd!());
            cpu.regs.set_wz_h(cpu.regs.a);
            goto!(921);
        }
        921 => {
            goto!(922);
        }
        922 => {
            iowrite!(cpu.regs.wz, cpu.regs.a);
            goto!(923);
        }
        923 => {
            wait!();
            cpu.regs.set_wz_l(cpu.regs.wz_l().wrapping_add(1));
            goto!(924);
        }
        924 => {
            goto!(925);
        }
        925 => {
            fetch!();
        }
        926 => {
            wait!();
            mread!(pc_postinc!());
            goto!(927);
        }
        927 => {
            cpu.regs.set_wz_l(gd!());
            goto!(928);
        }
        928 => {
            goto!(929);
        }
        929 => {
            wait!();
            mread!(pc_postinc!());
            goto!(930);
        }
        930 => {
            cpu.regs.set_wz_h(gd!());
            if !cc_nc!() {
                goto!(938);
            }
            goto!(931);
        }
        931 => {
            goto!(932);
        }
        932 => {
            goto!(933);
        }
        933 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(934);
        }
        934 => {
            goto!(935);
        }
        935 => {
            goto!(936);
        }
        936 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.pc = cpu.regs.wz;
            goto!(937);
        }
        937 => {
            goto!(938);
        }
        938 => {
            fetch!();
        }
        939 => {
            goto!(940);
        }
        940 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.d);
            goto!(941);
        }
        941 => {
            goto!(942);
        }
        942 => {
            goto!(943);
        }
        943 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.e);
            goto!(944);
        }
        944 => {
            goto!(945);
        }
        945 => {
            fetch!();
        }
        946 => {
            wait!();
            mread!(pc_postinc!());
            goto!(947);
        }
        947 => {
            cpu.state.dlatch = gd!();
            goto!(948);
        }
        948 => {
            sub8!(cpu.state.dlatch);
            fetch!();
        }
        949 => {
            goto!(950);
        }
        950 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(951);
        }
        951 => {
            goto!(952);
        }
        952 => {
            goto!(953);
        }
        953 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.wz = 0x10;
            cpu.regs.pc = cpu.regs.wz;
            goto!(954);
        }
        954 => {
            goto!(955);
        }
        955 => {
            fetch!();
        }
        956 => {
            goto!(957);
        }
        957 => {
            wait!();
            mread!(sp_postinc!());
            goto!(958);
        }
        958 => {
            cpu.regs.set_wz_l(gd!());
            goto!(959);
        }
        959 => {
            goto!(960);
        }
        960 => {
            wait!();
            mread!(sp_postinc!());
            goto!(961);
        }
        961 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(962);
        }
        962 => {
            fetch!();
        }
        963 => {
            wait!();
            mread!(pc_postinc!());
            goto!(964);
        }
        964 => {
            cpu.regs.set_wz_l(gd!());
            goto!(965);
        }
        965 => {
            goto!(966);
        }
        966 => {
            wait!();
            mread!(pc_postinc!());
            goto!(967);
        }
        967 => {
            cpu.regs.set_wz_h(gd!());
            if cc_c!() {
                cpu.regs.pc = cpu.regs.wz;
            }
            goto!(968);
        }
        968 => {
            fetch!();
        }
        969 => {
            wait!();
            mread!(pc_postinc!());
            goto!(970);
        }
        970 => {
            cpu.regs.set_wz_l(gd!());
            cpu.regs.set_wz_h(cpu.regs.a);
            goto!(971);
        }
        971 => {
            goto!(972);
        }
        972 => {
            goto!(973);
        }
        973 => {
            wait!();
            ioread!(wz_postinc!());
            goto!(974);
        }
        974 => {
            cpu.regs.a = gd!();
            goto!(975);
        }
        975 => {
            fetch!();
        }
        976 => {
            wait!();
            mread!(pc_postinc!());
            goto!(977);
        }
        977 => {
            cpu.regs.set_wz_l(gd!());
            goto!(978);
        }
        978 => {
            goto!(979);
        }
        979 => {
            wait!();
            mread!(pc_postinc!());
            goto!(980);
        }
        980 => {
            cpu.regs.set_wz_h(gd!());
            if !cc_c!() {
                goto!(988);
            }
            goto!(981);
        }
        981 => {
            goto!(982);
        }
        982 => {
            goto!(983);
        }
        983 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(984);
        }
        984 => {
            goto!(985);
        }
        985 => {
            goto!(986);
        }
        986 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.pc = cpu.regs.wz;
            goto!(987);
        }
        987 => {
            goto!(988);
        }
        988 => {
            fetch!();
        }
        989 => {
            wait!();
            mread!(pc_postinc!());
            goto!(990);
        }
        990 => {
            cpu.state.dlatch = gd!();
            goto!(991);
        }
        991 => {
            sbc8!(cpu.state.dlatch);
            fetch!();
        }
        992 => {
            goto!(993);
        }
        993 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(994);
        }
        994 => {
            goto!(995);
        }
        995 => {
            goto!(996);
        }
        996 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.wz = 0x18;
            cpu.regs.pc = cpu.regs.wz;
            goto!(997);
        }
        997 => {
            goto!(998);
        }
        998 => {
            fetch!();
        }
        999 => {
            goto!(1000);
        }
        1000 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1001);
        }
        1001 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1002);
        }
        1002 => {
            goto!(1003);
        }
        1003 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1004);
        }
        1004 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(1005);
        }
        1005 => {
            fetch!();
        }
        1006 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1007);
        }
        1007 => {
            cpu.regs.set_hlx_l(cpu.state.hlx_idx, gd!());
            goto!(1008);
        }
        1008 => {
            goto!(1009);
        }
        1009 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1010);
        }
        1010 => {
            cpu.regs.set_hlx_h(cpu.state.hlx_idx, gd!());
            goto!(1011);
        }
        1011 => {
            fetch!();
        }
        1012 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1013);
        }
        1013 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1014);
        }
        1014 => {
            goto!(1015);
        }
        1015 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1016);
        }
        1016 => {
            cpu.regs.set_wz_h(gd!());
            if cc_po!() {
                cpu.regs.pc = cpu.regs.wz;
            }
            goto!(1017);
        }
        1017 => {
            fetch!();
        }
        1018 => {
            wait!();
            mread!(cpu.regs.sp);
            goto!(1019);
        }
        1019 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1020);
        }
        1020 => {
            goto!(1021);
        }
        1021 => {
            wait!();
            mread!(cpu.regs.sp.wrapping_add(1));
            goto!(1022);
        }
        1022 => {
            cpu.regs.set_wz_h(gd!());
            goto!(1023);
        }
        1023 => {
            goto!(1024);
        }
        1024 => {
            goto!(1025);
        }
        1025 => {
            wait!();
            mwrite!(
                cpu.regs.sp.wrapping_add(1),
                cpu.regs.hlx_h(cpu.state.hlx_idx)
            );
            goto!(1026);
        }
        1026 => {
            goto!(1027);
        }
        1027 => {
            goto!(1028);
        }
        1028 => {
            wait!();
            mwrite!(cpu.regs.sp, cpu.regs.hlx_l(cpu.state.hlx_idx));
            cpu.regs.set_hlx(cpu.state.hlx_idx, cpu.regs.wz);
            goto!(1029);
        }
        1029 => {
            goto!(1030);
        }
        1030 => {
            goto!(1031);
        }
        1031 => {
            goto!(1032);
        }
        1032 => {
            fetch!();
        }
        1033 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1034);
        }
        1034 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1035);
        }
        1035 => {
            goto!(1036);
        }
        1036 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1037);
        }
        1037 => {
            cpu.regs.set_wz_h(gd!());
            if !cc_po!() {
                goto!(1045);
            }
            goto!(1038);
        }
        1038 => {
            goto!(1039);
        }
        1039 => {
            goto!(1040);
        }
        1040 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1041);
        }
        1041 => {
            goto!(1042);
        }
        1042 => {
            goto!(1043);
        }
        1043 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.pc = cpu.regs.wz;
            goto!(1044);
        }
        1044 => {
            goto!(1045);
        }
        1045 => {
            fetch!();
        }
        1046 => {
            goto!(1047);
        }
        1047 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.hlx_h(cpu.state.hlx_idx));
            goto!(1048);
        }
        1048 => {
            goto!(1049);
        }
        1049 => {
            goto!(1050);
        }
        1050 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.hlx_l(cpu.state.hlx_idx));
            goto!(1051);
        }
        1051 => {
            goto!(1052);
        }
        1052 => {
            fetch!();
        }
        1053 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1054);
        }
        1054 => {
            cpu.state.dlatch = gd!();
            goto!(1055);
        }
        1055 => {
            and8!(cpu.state.dlatch);
            fetch!();
        }
        1056 => {
            goto!(1057);
        }
        1057 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1058);
        }
        1058 => {
            goto!(1059);
        }
        1059 => {
            goto!(1060);
        }
        1060 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.wz = 0x20;
            cpu.regs.pc = cpu.regs.wz;
            goto!(1061);
        }
        1061 => {
            goto!(1062);
        }
        1062 => {
            fetch!();
        }
        1063 => {
            goto!(1064);
        }
        1064 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1065);
        }
        1065 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1066);
        }
        1066 => {
            goto!(1067);
        }
        1067 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1068);
        }
        1068 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(1069);
        }
        1069 => {
            fetch!();
        }
        1070 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1071);
        }
        1071 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1072);
        }
        1072 => {
            goto!(1073);
        }
        1073 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1074);
        }
        1074 => {
            cpu.regs.set_wz_h(gd!());
            if cc_pe!() {
                cpu.regs.pc = cpu.regs.wz;
            }
            goto!(1075);
        }
        1075 => {
            fetch!();
        }
        1076 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1077);
        }
        1077 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1078);
        }
        1078 => {
            goto!(1079);
        }
        1079 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1080);
        }
        1080 => {
            cpu.regs.set_wz_h(gd!());
            if !cc_pe!() {
                goto!(1088);
            }
            goto!(1081);
        }
        1081 => {
            goto!(1082);
        }
        1082 => {
            goto!(1083);
        }
        1083 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1084);
        }
        1084 => {
            goto!(1085);
        }
        1085 => {
            goto!(1086);
        }
        1086 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.pc = cpu.regs.wz;
            goto!(1087);
        }
        1087 => {
            goto!(1088);
        }
        1088 => {
            fetch!();
        }
        1089 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1090);
        }
        1090 => {
            cpu.state.dlatch = gd!();
            goto!(1091);
        }
        1091 => {
            xor8!(cpu.state.dlatch);
            fetch!();
        }
        1092 => {
            goto!(1093);
        }
        1093 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1094);
        }
        1094 => {
            goto!(1095);
        }
        1095 => {
            goto!(1096);
        }
        1096 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.wz = 0x28;
            cpu.regs.pc = cpu.regs.wz;
            goto!(1097);
        }
        1097 => {
            goto!(1098);
        }
        1098 => {
            fetch!();
        }
        1099 => {
            goto!(1100);
        }
        1100 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1101);
        }
        1101 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1102);
        }
        1102 => {
            goto!(1103);
        }
        1103 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1104);
        }
        1104 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(1105);
        }
        1105 => {
            fetch!();
        }
        1106 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1107);
        }
        1107 => {
            cpu.regs.f = Flags::from_bits_retain(gd!());
            goto!(1108);
        }
        1108 => {
            goto!(1109);
        }
        1109 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1110);
        }
        1110 => {
            cpu.regs.a = gd!();
            goto!(1111);
        }
        1111 => {
            fetch!();
        }
        1112 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1113);
        }
        1113 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1114);
        }
        1114 => {
            goto!(1115);
        }
        1115 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1116);
        }
        1116 => {
            cpu.regs.set_wz_h(gd!());
            if cc_p!() {
                cpu.regs.pc = cpu.regs.wz;
            }
            goto!(1117);
        }
        1117 => {
            fetch!();
        }
        1118 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1119);
        }
        1119 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1120);
        }
        1120 => {
            goto!(1121);
        }
        1121 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1122);
        }
        1122 => {
            cpu.regs.set_wz_h(gd!());
            if !cc_p!() {
                goto!(1130);
            }
            goto!(1123);
        }
        1123 => {
            goto!(1124);
        }
        1124 => {
            goto!(1125);
        }
        1125 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1126);
        }
        1126 => {
            goto!(1127);
        }
        1127 => {
            goto!(1128);
        }
        1128 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.pc = cpu.regs.wz;
            goto!(1129);
        }
        1129 => {
            goto!(1130);
        }
        1130 => {
            fetch!();
        }
        1131 => {
            goto!(1132);
        }
        1132 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.a);
            goto!(1133);
        }
        1133 => {
            goto!(1134);
        }
        1134 => {
            goto!(1135);
        }
        1135 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.f.bits());
            goto!(1136);
        }
        1136 => {
            goto!(1137);
        }
        1137 => {
            fetch!();
        }
        1138 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1139);
        }
        1139 => {
            cpu.state.dlatch = gd!();
            goto!(1140);
        }
        1140 => {
            or8!(cpu.state.dlatch);
            fetch!();
        }
        1141 => {
            goto!(1142);
        }
        1142 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1143);
        }
        1143 => {
            goto!(1144);
        }
        1144 => {
            goto!(1145);
        }
        1145 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.wz = 0x30;
            cpu.regs.pc = cpu.regs.wz;
            goto!(1146);
        }
        1146 => {
            goto!(1147);
        }
        1147 => {
            fetch!();
        }
        1148 => {
            goto!(1149);
        }
        1149 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1150);
        }
        1150 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1151);
        }
        1151 => {
            goto!(1152);
        }
        1152 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1153);
        }
        1153 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(1154);
        }
        1154 => {
            fetch!();
        }
        1155 => {
            goto!(1156);
        }
        1156 => {
            fetch!();
        }
        1157 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1158);
        }
        1158 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1159);
        }
        1159 => {
            goto!(1160);
        }
        1160 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1161);
        }
        1161 => {
            cpu.regs.set_wz_h(gd!());
            if cc_m!() {
                cpu.regs.pc = cpu.regs.wz;
            }
            goto!(1162);
        }
        1162 => {
            fetch!();
        }
        1163 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1164);
        }
        1164 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1165);
        }
        1165 => {
            goto!(1166);
        }
        1166 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1167);
        }
        1167 => {
            cpu.regs.set_wz_h(gd!());
            if !cc_m!() {
                goto!(1175);
            }
            goto!(1168);
        }
        1168 => {
            goto!(1169);
        }
        1169 => {
            goto!(1170);
        }
        1170 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1171);
        }
        1171 => {
            goto!(1172);
        }
        1172 => {
            goto!(1173);
        }
        1173 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.pc = cpu.regs.wz;
            goto!(1174);
        }
        1174 => {
            goto!(1175);
        }
        1175 => {
            fetch!();
        }
        1176 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1177);
        }
        1177 => {
            cpu.state.dlatch = gd!();
            goto!(1178);
        }
        1178 => {
            cp8!(cpu.state.dlatch);
            fetch!();
        }
        1179 => {
            goto!(1180);
        }
        1180 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1181);
        }
        1181 => {
            goto!(1182);
        }
        1182 => {
            goto!(1183);
        }
        1183 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.wz = 0x38;
            cpu.regs.pc = cpu.regs.wz;
            goto!(1184);
        }
        1184 => {
            goto!(1185);
        }
        1185 => {
            fetch!();
        }
        1186 => {
            goto!(1187);
        }
        1187 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1188);
        }
        1188 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1189);
        }
        1189 => {
            cpu.regs.b = in8!(cpu.state.dlatch);
            fetch!();
        }
        1190 => {
            iowrite!(cpu.regs.bc(), cpu.regs.b);
            goto!(1191);
        }
        1191 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1192);
        }
        1192 => {
            goto!(1193);
        }
        1193 => {
            fetch!();
        }
        1194 => {
            goto!(1195);
        }
        1195 => {
            goto!(1196);
        }
        1196 => {
            goto!(1197);
        }
        1197 => {
            goto!(1198);
        }
        1198 => {
            goto!(1199);
        }
        1199 => {
            goto!(1200);
        }
        1200 => {
            fetch!();
        }
        1201 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1202);
        }
        1202 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1203);
        }
        1203 => {
            goto!(1204);
        }
        1204 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1205);
        }
        1205 => {
            cpu.regs.set_wz_h(gd!());
            goto!(1206);
        }
        1206 => {
            goto!(1207);
        }
        1207 => {
            wait!();
            mwrite!(wz_postinc!(), cpu.regs.c);
            goto!(1208);
        }
        1208 => {
            goto!(1209);
        }
        1209 => {
            goto!(1210);
        }
        1210 => {
            wait!();
            mwrite!(cpu.regs.wz, cpu.regs.b);
            goto!(1211);
        }
        1211 => {
            goto!(1212);
        }
        1212 => {
            fetch!();
        }
        1213 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1214);
        }
        1214 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1215);
        }
        1215 => {
            goto!(1216);
        }
        1216 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1217);
        }
        1217 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(1218);
        }
        1218 => {
            pins = cpu.fetch(pins);
            cpu.state.iff1 = cpu.state.iff2;
            pins
        }
        1219 => {
            cpu.regs.i = cpu.regs.a;
            fetch!();
        }
        1220 => {
            goto!(1221);
        }
        1221 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1222);
        }
        1222 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1223);
        }
        1223 => {
            cpu.regs.c = in8!(cpu.state.dlatch);
            fetch!();
        }
        1224 => {
            iowrite!(cpu.regs.bc(), cpu.regs.c);
            goto!(1225);
        }
        1225 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1226);
        }
        1226 => {
            goto!(1227);
        }
        1227 => {
            fetch!();
        }
        1228 => {
            goto!(1229);
        }
        1229 => {
            goto!(1230);
        }
        1230 => {
            goto!(1231);
        }
        1231 => {
            goto!(1232);
        }
        1232 => {
            goto!(1233);
        }
        1233 => {
            goto!(1234);
        }
        1234 => {
            fetch!();
        }
        1235 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1236);
        }
        1236 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1237);
        }
        1237 => {
            goto!(1238);
        }
        1238 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1239);
        }
        1239 => {
            cpu.regs.set_wz_h(gd!());
            goto!(1240);
        }
        1240 => {
            goto!(1241);
        }
        1241 => {
            wait!();
            mread!(wz_postinc!());
            goto!(1242);
        }
        1242 => {
            cpu.regs.c = gd!();
            goto!(1243);
        }
        1243 => {
            goto!(1244);
        }
        1244 => {
            wait!();
            mread!(cpu.regs.wz);
            goto!(1245);
        }
        1245 => {
            cpu.regs.b = gd!();
            goto!(1246);
        }
        1246 => {
            fetch!();
        }
        1247 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1248);
        }
        1248 => {
            cpu.regs.set_wz_l(gd!());
            pins |= pins::RETI;
            goto!(1249);
        }
        1249 => {
            goto!(1250);
        }
        1250 => {
            wait!();
            mread!(sp_postinc!());
            goto!(1251);
        }
        1251 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.pc = cpu.regs.wz;
            goto!(1252);
        }
        1252 | 1281 | 1310 | 1339 | 1377 | 1415 | 1443 => {
            pins = cpu.fetch(pins);
            cpu.state.iff1 = cpu.state.iff2;
            pins
        }
        1253 => {
            cpu.regs.r = cpu.regs.a;
            fetch!();
        }
        1254 => {
            goto!(1255);
        }
        1255 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1256);
        }
        1256 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1257);
        }
        1257 => {
            cpu.regs.d = in8!(cpu.state.dlatch);
            fetch!();
        }
        1258 => {
            iowrite!(cpu.regs.bc(), cpu.regs.d);
            goto!(1259);
        }
        1259 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1260);
        }
        1260 => {
            goto!(1261);
        }
        1261 => {
            fetch!();
        }
        1262 => {
            goto!(1263);
        }
        1263 => {
            goto!(1264);
        }
        1264 => {
            goto!(1265);
        }
        1265 => {
            goto!(1266);
        }
        1266 => {
            goto!(1267);
        }
        1267 => {
            goto!(1268);
        }
        1268 => {
            fetch!();
        }
        1269 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1270);
        }
        1270 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1271);
        }
        1271 => {
            goto!(1272);
        }
        1272 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1273);
        }
        1273 => {
            cpu.regs.set_wz_h(gd!());
            goto!(1274);
        }
        1274 => {
            goto!(1275);
        }
        1275 => {
            wait!();
            mwrite!(wz_postinc!(), cpu.regs.e);
            goto!(1276);
        }
        1276 => {
            goto!(1277);
        }
        1277 => {
            goto!(1278);
        }
        1278 => {
            wait!();
            mwrite!(cpu.regs.wz, cpu.regs.d);
            goto!(1279);
        }
        1279 => {
            goto!(1280);
        }
        1280 => {
            fetch!();
        }
        1282 => {
            cpu.regs.a = cpu.regs.i;
            cpu.regs.f = Flags::from_bits_retain(crate::alu::sziff2_flags(cpu, cpu.regs.i));
            fetch!();
        }
        1283 => {
            goto!(1284);
        }
        1284 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1285);
        }
        1285 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1286);
        }
        1286 => {
            cpu.regs.e = in8!(cpu.state.dlatch);
            fetch!();
        }
        1287 => {
            iowrite!(cpu.regs.bc(), cpu.regs.e);
            goto!(1288);
        }
        1288 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1289);
        }
        1289 => {
            goto!(1290);
        }
        1290 => {
            fetch!();
        }
        1291 => {
            goto!(1292);
        }
        1292 => {
            goto!(1293);
        }
        1293 => {
            goto!(1294);
        }
        1294 => {
            goto!(1295);
        }
        1295 => {
            goto!(1296);
        }
        1296 => {
            goto!(1297);
        }
        1297 => {
            fetch!();
        }
        1298 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1299);
        }
        1299 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1300);
        }
        1300 => {
            goto!(1301);
        }
        1301 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1302);
        }
        1302 => {
            cpu.regs.set_wz_h(gd!());
            goto!(1303);
        }
        1303 => {
            goto!(1304);
        }
        1304 => {
            wait!();
            mread!(wz_postinc!());
            goto!(1305);
        }
        1305 => {
            cpu.regs.e = gd!();
            goto!(1306);
        }
        1306 => {
            goto!(1307);
        }
        1307 => {
            wait!();
            mread!(cpu.regs.wz);
            goto!(1308);
        }
        1308 => {
            cpu.regs.d = gd!();
            goto!(1309);
        }
        1309 => {
            fetch!();
        }
        1311 => {
            cpu.regs.a = cpu.regs.r;
            cpu.regs.f = Flags::from_bits_retain(crate::alu::sziff2_flags(cpu, cpu.regs.r));
            fetch!();
        }
        1312 => {
            goto!(1313);
        }
        1313 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1314);
        }
        1314 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1315);
        }
        1315 => {
            cpu.regs.h = in8!(cpu.state.dlatch);
            fetch!();
        }
        1316 => {
            iowrite!(cpu.regs.bc(), cpu.regs.h);
            goto!(1317);
        }
        1317 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1318);
        }
        1318 => {
            goto!(1319);
        }
        1319 => {
            fetch!();
        }
        1320 => {
            goto!(1321);
        }
        1321 => {
            goto!(1322);
        }
        1322 => {
            goto!(1323);
        }
        1323 => {
            goto!(1324);
        }
        1324 => {
            goto!(1325);
        }
        1325 => {
            goto!(1326);
        }
        1326 => {
            fetch!();
        }
        1327 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1328);
        }
        1328 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1329);
        }
        1329 => {
            goto!(1330);
        }
        1330 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1331);
        }
        1331 => {
            cpu.regs.set_wz_h(gd!());
            goto!(1332);
        }
        1332 => {
            goto!(1333);
        }
        1333 => {
            wait!();
            mwrite!(wz_postinc!(), cpu.regs.l);
            goto!(1334);
        }
        1334 => {
            goto!(1335);
        }
        1335 => {
            goto!(1336);
        }
        1336 => {
            wait!();
            mwrite!(cpu.regs.wz, cpu.regs.h);
            goto!(1337);
        }
        1337 => {
            goto!(1338);
        }
        1338 => {
            fetch!();
        }
        1340 => {
            wait!();
            mread!(cpu.regs.hl());
            goto!(1341);
        }
        1341 => {
            cpu.state.dlatch = gd!();
            goto!(1342);
        }
        1342 => {
            cpu.state.dlatch = crate::alu::rrd(cpu, cpu.state.dlatch);
            goto!(1343);
        }
        1343 => {
            goto!(1344);
        }
        1344 => {
            goto!(1345);
        }
        1345 => {
            goto!(1346);
        }
        1346 => {
            goto!(1347);
        }
        1347 => {
            wait!();
            mwrite!(cpu.regs.hl(), cpu.state.dlatch);
            cpu.regs.wz = cpu.regs.hl().wrapping_add(1);
            goto!(1348);
        }
        1348 => {
            goto!(1349);
        }
        1349 => {
            fetch!();
        }
        1350 => {
            goto!(1351);
        }
        1351 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1352);
        }
        1352 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1353);
        }
        1353 => {
            cpu.regs.l = in8!(cpu.state.dlatch);
            fetch!();
        }
        1354 => {
            iowrite!(cpu.regs.bc(), cpu.regs.l);
            goto!(1355);
        }
        1355 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1356);
        }
        1356 => {
            goto!(1357);
        }
        1357 => {
            fetch!();
        }
        1358 => {
            goto!(1359);
        }
        1359 => {
            goto!(1360);
        }
        1360 => {
            goto!(1361);
        }
        1361 => {
            goto!(1362);
        }
        1362 => {
            goto!(1363);
        }
        1363 => {
            goto!(1364);
        }
        1364 => {
            fetch!();
        }
        1365 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1366);
        }
        1366 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1367);
        }
        1367 => {
            goto!(1368);
        }
        1368 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1369);
        }
        1369 => {
            cpu.regs.set_wz_h(gd!());
            goto!(1370);
        }
        1370 => {
            goto!(1371);
        }
        1371 => {
            wait!();
            mread!(wz_postinc!());
            goto!(1372);
        }
        1372 => {
            cpu.regs.l = gd!();
            goto!(1373);
        }
        1373 => {
            goto!(1374);
        }
        1374 => {
            wait!();
            mread!(cpu.regs.wz);
            goto!(1375);
        }
        1375 => {
            cpu.regs.h = gd!();
            goto!(1376);
        }
        1376 => {
            fetch!();
        }
        1378 => {
            wait!();
            mread!(cpu.regs.hl());
            goto!(1379);
        }
        1379 => {
            cpu.state.dlatch = gd!();
            goto!(1380);
        }
        1380 => {
            cpu.state.dlatch = crate::alu::rld(cpu, cpu.state.dlatch);
            goto!(1381);
        }
        1381 => {
            goto!(1382);
        }
        1382 => {
            goto!(1383);
        }
        1383 => {
            goto!(1384);
        }
        1384 => {
            goto!(1385);
        }
        1385 => {
            wait!();
            mwrite!(cpu.regs.hl(), cpu.state.dlatch);
            cpu.regs.wz = cpu.regs.hl().wrapping_add(1);
            goto!(1386);
        }
        1386 => {
            goto!(1387);
        }
        1387 => {
            fetch!();
        }
        1388 => {
            goto!(1389);
        }
        1389 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1390);
        }
        1390 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1391);
        }
        1391 => {
            in8!(cpu.state.dlatch);
            fetch!();
        }
        1392 => {
            iowrite!(cpu.regs.bc(), 0);
            goto!(1393);
        }
        1393 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1394);
        }
        1394 => {
            goto!(1395);
        }
        1395 => {
            fetch!();
        }
        1396 => {
            goto!(1397);
        }
        1397 => {
            goto!(1398);
        }
        1398 => {
            goto!(1399);
        }
        1399 => {
            goto!(1400);
        }
        1400 => {
            goto!(1401);
        }
        1401 => {
            goto!(1402);
        }
        1402 => {
            fetch!();
        }
        1403 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1404);
        }
        1404 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1405);
        }
        1405 => {
            goto!(1406);
        }
        1406 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1407);
        }
        1407 => {
            cpu.regs.set_wz_h(gd!());
            goto!(1408);
        }
        1408 => {
            goto!(1409);
        }
        1409 => {
            wait!();
            mwrite!(wz_postinc!(), cpu.regs.sp_l());
            goto!(1410);
        }
        1410 => {
            goto!(1411);
        }
        1411 => {
            goto!(1412);
        }
        1412 => {
            wait!();
            mwrite!(cpu.regs.wz, cpu.regs.sp_h());
            goto!(1413);
        }
        1413 => {
            goto!(1414);
        }
        1414 => {
            fetch!();
        }
        1416 => {
            goto!(1417);
        }
        1417 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1418);
        }
        1418 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1419);
        }
        1419 => {
            cpu.regs.a = in8!(cpu.state.dlatch);
            fetch!();
        }
        1420 => {
            iowrite!(cpu.regs.bc(), cpu.regs.a);
            goto!(1421);
        }
        1421 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            goto!(1422);
        }
        1422 => {
            goto!(1423);
        }
        1423 => {
            fetch!();
        }
        1424 => {
            goto!(1425);
        }
        1425 => {
            goto!(1426);
        }
        1426 => {
            goto!(1427);
        }
        1427 => {
            goto!(1428);
        }
        1428 => {
            goto!(1429);
        }
        1429 => {
            goto!(1430);
        }
        1430 => {
            fetch!();
        }
        1431 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1432);
        }
        1432 => {
            cpu.regs.set_wz_l(gd!());
            goto!(1433);
        }
        1433 => {
            goto!(1434);
        }
        1434 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1435);
        }
        1435 => {
            cpu.regs.set_wz_h(gd!());
            goto!(1436);
        }
        1436 => {
            goto!(1437);
        }
        1437 => {
            wait!();
            mread!(wz_postinc!());
            goto!(1438);
        }
        1438 => {
            cpu.regs.set_sp_l(gd!());
            goto!(1439);
        }
        1439 => {
            goto!(1440);
        }
        1440 => {
            wait!();
            mread!(cpu.regs.wz);
            goto!(1441);
        }
        1441 => {
            cpu.regs.set_sp_h(gd!());
            goto!(1442);
        }
        1442 => {
            fetch!();
        }
        1444 => {
            wait!();
            mread!(hl_postinc!());
            goto!(1445);
        }
        1445 => {
            cpu.state.dlatch = gd!();
            goto!(1446);
        }
        1446 => {
            goto!(1447);
        }
        1447 => {
            wait!();
            mwrite!(de_postinc!(), cpu.state.dlatch);
            goto!(1448);
        }
        1448 => {
            goto!(1449);
        }
        1449 => {
            crate::alu::ldi_ldd_flags(cpu, cpu.state.dlatch);
            goto!(1450);
        }
        1450 => {
            goto!(1451);
        }
        1451 => {
            fetch!();
        }
        1452 => {
            wait!();
            mread!(hl_postinc!());
            goto!(1453);
        }
        1453 => {
            cpu.state.dlatch = gd!();
            goto!(1454);
        }
        1454 => {
            cpu.regs.wz = cpu.regs.wz.wrapping_add(1);
            crate::alu::cpi_cpd_flags(cpu, cpu.state.dlatch);
            goto!(1455);
        }
        1455 => {
            goto!(1456);
        }
        1456 => {
            goto!(1457);
        }
        1457 => {
            goto!(1458);
        }
        1458 => {
            goto!(1459);
        }
        1459 => {
            fetch!();
        }
        1460 => {
            goto!(1461);
        }
        1461 => {
            goto!(1462);
        }
        1462 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1463);
        }
        1463 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            cpu.regs.b = cpu.regs.b.wrapping_sub(1);
            goto!(1464);
        }
        1464 => {
            goto!(1465);
        }
        1465 => {
            wait!();
            mwrite!(hl_postinc!(), cpu.state.dlatch);
            crate::alu::ini_ind_flags(cpu, cpu.state.dlatch, cpu.regs.c.wrapping_add(1));
            goto!(1466);
        }
        1466 => {
            goto!(1467);
        }
        1467 => {
            fetch!();
        }
        1468 => {
            goto!(1469);
        }
        1469 => {
            wait!();
            mread!(hl_postinc!());
            goto!(1470);
        }
        1470 => {
            cpu.state.dlatch = gd!();
            cpu.regs.b = cpu.regs.b.wrapping_sub(1);
            goto!(1471);
        }
        1471 => {
            goto!(1472);
        }
        1472 => {
            iowrite!(cpu.regs.bc(), cpu.state.dlatch);
            goto!(1473);
        }
        1473 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            crate::alu::outi_outd_flags(cpu, cpu.state.dlatch);
            goto!(1474);
        }
        1474 => {
            goto!(1475);
        }
        1475 => {
            fetch!();
        }
        1476 => {
            wait!();
            mread!(hl_postdec!());
            goto!(1477);
        }
        1477 => {
            cpu.state.dlatch = gd!();
            goto!(1478);
        }
        1478 => {
            goto!(1479);
        }
        1479 => {
            wait!();
            mwrite!(de_postdec!(), cpu.state.dlatch);
            goto!(1480);
        }
        1480 => {
            goto!(1481);
        }
        1481 => {
            crate::alu::ldi_ldd_flags(cpu, cpu.state.dlatch);
            goto!(1482);
        }
        1482 => {
            goto!(1483);
        }
        1483 => {
            fetch!();
        }
        1484 => {
            wait!();
            mread!(hl_postdec!());
            goto!(1485);
        }
        1485 => {
            cpu.state.dlatch = gd!();
            goto!(1486);
        }
        1486 => {
            cpu.regs.wz = cpu.regs.wz.wrapping_sub(1);
            crate::alu::cpi_cpd_flags(cpu, cpu.state.dlatch);
            goto!(1487);
        }
        1487 => {
            goto!(1488);
        }
        1488 => {
            goto!(1489);
        }
        1489 => {
            goto!(1490);
        }
        1490 => {
            goto!(1491);
        }
        1491 => {
            fetch!();
        }
        1492 => {
            goto!(1493);
        }
        1493 => {
            goto!(1494);
        }
        1494 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1495);
        }
        1495 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_sub(1);
            cpu.regs.b = cpu.regs.b.wrapping_sub(1);
            goto!(1496);
        }
        1496 => {
            goto!(1497);
        }
        1497 => {
            wait!();
            mwrite!(hl_postdec!(), cpu.state.dlatch);
            crate::alu::ini_ind_flags(cpu, cpu.state.dlatch, cpu.regs.c.wrapping_sub(1));
            goto!(1498);
        }
        1498 => {
            goto!(1499);
        }
        1499 => {
            fetch!();
        }
        1500 => {
            goto!(1501);
        }
        1501 => {
            wait!();
            mread!(hl_postdec!());
            goto!(1502);
        }
        1502 => {
            cpu.state.dlatch = gd!();
            cpu.regs.b = cpu.regs.b.wrapping_sub(1);
            goto!(1503);
        }
        1503 => {
            goto!(1504);
        }
        1504 => {
            iowrite!(cpu.regs.bc(), cpu.state.dlatch);
            goto!(1505);
        }
        1505 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_sub(1);
            crate::alu::outi_outd_flags(cpu, cpu.state.dlatch);
            goto!(1506);
        }
        1506 => {
            goto!(1507);
        }
        1507 => {
            fetch!();
        }
        1508 => {
            wait!();
            mread!(hl_postinc!());
            goto!(1509);
        }
        1509 => {
            cpu.state.dlatch = gd!();
            goto!(1510);
        }
        1510 => {
            goto!(1511);
        }
        1511 => {
            wait!();
            mwrite!(de_postinc!(), cpu.state.dlatch);
            goto!(1512);
        }
        1512 => {
            goto!(1513);
        }
        1513 => {
            if !crate::alu::ldi_ldd_flags(cpu, cpu.state.dlatch) {
                goto!(1519);
            }
            goto!(1514);
        }
        1514 => {
            goto!(1515);
        }
        1515 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_sub(2);
            cpu.regs.wz = cpu.regs.pc.wrapping_add(1);
            goto!(1516);
        }
        1516 => {
            goto!(1517);
        }
        1517 => {
            goto!(1518);
        }
        1518 => {
            goto!(1519);
        }
        1519 => {
            goto!(1520);
        }
        1520 => {
            fetch!();
        }
        1521 => {
            wait!();
            mread!(hl_postinc!());
            goto!(1522);
        }
        1522 => {
            cpu.state.dlatch = gd!();
            goto!(1523);
        }
        1523 => {
            cpu.regs.wz = cpu.regs.wz.wrapping_add(1);
            if !crate::alu::cpi_cpd_flags(cpu, cpu.state.dlatch) {
                goto!(1529);
            }
            goto!(1524);
        }
        1524 => {
            goto!(1525);
        }
        1525 => {
            goto!(1526);
        }
        1526 => {
            goto!(1527);
        }
        1527 => {
            goto!(1528);
        }
        1528 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_sub(2);
            cpu.regs.wz = cpu.regs.pc.wrapping_add(1);
            goto!(1529);
        }
        1529 => {
            goto!(1530);
        }
        1530 => {
            goto!(1531);
        }
        1531 => {
            goto!(1532);
        }
        1532 => {
            goto!(1533);
        }
        1533 => {
            fetch!();
        }
        1534 => {
            goto!(1535);
        }
        1535 => {
            goto!(1536);
        }
        1536 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1537);
        }
        1537 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            cpu.regs.b = cpu.regs.b.wrapping_sub(1);
            goto!(1538);
        }
        1538 => {
            goto!(1539);
        }
        1539 => {
            wait!();
            mwrite!(hl_postinc!(), cpu.state.dlatch);
            if !crate::alu::ini_ind_flags(cpu, cpu.state.dlatch, cpu.regs.c.wrapping_add(1)) {
                goto!(1545);
            }
            goto!(1540);
        }
        1540 => {
            goto!(1541);
        }
        1541 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_sub(2);
            cpu.regs.wz = cpu.regs.pc.wrapping_add(1);
            goto!(1542);
        }
        1542 => {
            goto!(1543);
        }
        1543 => {
            goto!(1544);
        }
        1544 => {
            goto!(1545);
        }
        1545 => {
            goto!(1546);
        }
        1546 => {
            fetch!();
        }
        1547 => {
            goto!(1548);
        }
        1548 => {
            wait!();
            mread!(hl_postinc!());
            goto!(1549);
        }
        1549 => {
            cpu.state.dlatch = gd!();
            cpu.regs.b = cpu.regs.b.wrapping_sub(1);
            goto!(1550);
        }
        1550 => {
            goto!(1551);
        }
        1551 => {
            iowrite!(cpu.regs.bc(), cpu.state.dlatch);
            goto!(1552);
        }
        1552 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_add(1);
            if !crate::alu::outi_outd_flags(cpu, cpu.state.dlatch) {
                goto!(1558);
            }
            goto!(1553);
        }
        1553 => {
            goto!(1554);
        }
        1554 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_sub(2);
            cpu.regs.wz = cpu.regs.pc.wrapping_add(1);
            goto!(1555);
        }
        1555 => {
            goto!(1556);
        }
        1556 => {
            goto!(1557);
        }
        1557 => {
            goto!(1558);
        }
        1558 => {
            goto!(1559);
        }
        1559 => {
            fetch!();
        }
        1560 => {
            wait!();
            mread!(hl_postdec!());
            goto!(1561);
        }
        1561 => {
            cpu.state.dlatch = gd!();
            goto!(1562);
        }
        1562 => {
            goto!(1563);
        }
        1563 => {
            wait!();
            mwrite!(de_postdec!(), cpu.state.dlatch);
            goto!(1564);
        }
        1564 => {
            goto!(1565);
        }
        1565 => {
            if !crate::alu::ldi_ldd_flags(cpu, cpu.state.dlatch) {
                goto!(1571);
            }
            goto!(1566);
        }
        1566 => {
            goto!(1567);
        }
        1567 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_sub(2);
            cpu.regs.wz = cpu.regs.pc.wrapping_add(1);
            goto!(1568);
        }
        1568 => {
            goto!(1569);
        }
        1569 => {
            goto!(1570);
        }
        1570 => {
            goto!(1571);
        }
        1571 => {
            goto!(1572);
        }
        1572 => {
            fetch!();
        }
        1573 => {
            wait!();
            mread!(hl_postdec!());
            goto!(1574);
        }
        1574 => {
            cpu.state.dlatch = gd!();
            goto!(1575);
        }
        1575 => {
            cpu.regs.wz = cpu.regs.wz.wrapping_sub(1);
            if !crate::alu::cpi_cpd_flags(cpu, cpu.state.dlatch) {
                goto!(1581);
            }
            goto!(1576);
        }
        1576 => {
            goto!(1577);
        }
        1577 => {
            goto!(1578);
        }
        1578 => {
            goto!(1579);
        }
        1579 => {
            goto!(1580);
        }
        1580 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_sub(2);
            cpu.regs.wz = cpu.regs.pc.wrapping_add(1);
            goto!(1581);
        }
        1581 => {
            goto!(1582);
        }
        1582 => {
            goto!(1583);
        }
        1583 => {
            goto!(1584);
        }
        1584 => {
            goto!(1585);
        }
        1585 => {
            fetch!();
        }
        1586 => {
            goto!(1587);
        }
        1587 => {
            goto!(1588);
        }
        1588 => {
            wait!();
            ioread!(cpu.regs.bc());
            goto!(1589);
        }
        1589 => {
            cpu.state.dlatch = gd!();
            cpu.regs.wz = cpu.regs.bc().wrapping_sub(1);
            cpu.regs.b = cpu.regs.b.wrapping_sub(1);
            goto!(1590);
        }
        1590 => {
            goto!(1591);
        }
        1591 => {
            wait!();
            mwrite!(hl_postdec!(), cpu.state.dlatch);
            if !crate::alu::ini_ind_flags(cpu, cpu.state.dlatch, cpu.regs.c.wrapping_sub(1)) {
                goto!(1597);
            }
            goto!(1592);
        }
        1592 => {
            goto!(1593);
        }
        1593 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_sub(2);
            cpu.regs.wz = cpu.regs.pc.wrapping_add(1);
            goto!(1594);
        }
        1594 => {
            goto!(1595);
        }
        1595 => {
            goto!(1596);
        }
        1596 => {
            goto!(1597);
        }
        1597 => {
            goto!(1598);
        }
        1598 => {
            fetch!();
        }
        1599 => {
            goto!(1600);
        }
        1600 => {
            wait!();
            mread!(hl_postdec!());
            goto!(1601);
        }
        1601 => {
            cpu.state.dlatch = gd!();
            cpu.regs.b = cpu.regs.b.wrapping_sub(1);
            goto!(1602);
        }
        1602 => {
            goto!(1603);
        }
        1603 => {
            iowrite!(cpu.regs.bc(), cpu.state.dlatch);
            goto!(1604);
        }
        1604 => {
            wait!();
            cpu.regs.wz = cpu.regs.bc().wrapping_sub(1);
            if !crate::alu::outi_outd_flags(cpu, cpu.state.dlatch) {
                goto!(1610);
            }
            goto!(1605);
        }
        1605 => {
            goto!(1606);
        }
        1606 => {
            cpu.regs.pc = cpu.regs.pc.wrapping_sub(2);
            cpu.regs.wz = cpu.regs.pc.wrapping_add(1);
            goto!(1607);
        }
        1607 => {
            goto!(1608);
        }
        1608 => {
            goto!(1609);
        }
        1609 => {
            goto!(1610);
        }
        1610 => {
            goto!(1611);
        }
        1611 => {
            fetch!();
        }
        1612 => {
            let z = cpu.state.opcode & 7;
            crate::alu::cb_action(cpu, z, z);
            fetch!();
        }
        1613 => {
            goto!(1614);
        }
        1614 => {
            wait!();
            mread!(cpu.regs.hl());
            goto!(1615);
        }
        1615 => {
            cpu.state.dlatch = gd!();
            if !crate::alu::cb_action(cpu, 6, 6) {
                goto!(1619);
            }
            goto!(1616);
        }
        1616 => {
            goto!(1617);
        }
        1617 => {
            goto!(1618);
        }
        1618 => {
            wait!();
            mwrite!(cpu.regs.hl(), cpu.state.dlatch);
            goto!(1619);
        }
        1619 => {
            goto!(1620);
        }
        1620 => {
            fetch!();
        }
        1621 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1622);
        }
        1622 => {
            let d = gd!() as i8 as u16;
            cpu.state.addr = cpu.regs.hlx(cpu.state.hlx_idx).wrapping_add(d);
            cpu.regs.wz = cpu.state.addr;
            goto!(1623);
        }
        1623 => {
            goto!(1624);
        }
        1624 => {
            wait!();
            mread!(pc_postinc!());
            goto!(1625);
        }
        1625 => {
            cpu.state.opcode = gd!();
            goto!(1626);
        }
        1626 => {
            goto!(1627);
        }
        1627 => {
            goto!(1628);
        }
        1628 => {
            goto!(1629);
        }
        1629 => {
            wait!();
            mread!(cpu.state.addr);
            goto!(1630);
        }
        1630 => {
            cpu.state.dlatch = gd!();
            if !crate::alu::cb_action(cpu, 6, cpu.state.opcode & 7) {
                goto!(1634);
            }
            goto!(1631);
        }
        1631 => {
            goto!(1632);
        }
        1632 => {
            goto!(1633);
        }
        1633 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.state.dlatch);
            goto!(1634);
        }
        1634 => {
            goto!(1635);
        }
        1635 => {
            fetch!();
        }
        1636 => {
            cpu.state.iff1 = false;
            cpu.state.iff2 = false;
            goto!(1637);
        }
        1637 => {
            pins |= pins::M1 | pins::IORQ;
            goto!(1638);
        }
        1638 => {
            wait!();
            cpu.state.opcode = gd!();
            goto!(1639);
        }
        1639 => {
            pins = cpu.refresh(pins);
            goto!(1640);
        }
        1640 => {
            cpu.state.addr = cpu.regs.hl();
            goto!(cpu.state.opcode as u16);
        }
        1641 => {
            fetch!();
        }
        1642 => {
            cpu.state.iff1 = false;
            cpu.state.iff2 = false;
            goto!(1643);
        }
        1643 => {
            pins |= pins::M1 | pins::IORQ;
            goto!(1644);
        }
        1644 => {
            wait!();
            goto!(1645);
        }
        1645 => {
            pins = cpu.refresh(pins);
            goto!(1646);
        }
        1646 => {
            goto!(1647);
        }
        1647 => {
            goto!(1648);
        }
        1648 => {
            goto!(1649);
        }
        1649 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1650);
        }
        1650 => {
            goto!(1651);
        }
        1651 => {
            goto!(1652);
        }
        1652 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.wz = 0x0038;
            cpu.regs.pc = cpu.regs.wz;
            goto!(1653);
        }
        1653 => {
            goto!(1654);
        }
        1654 => {
            fetch!();
        }
        1655 => {
            cpu.state.iff1 = false;
            cpu.state.iff2 = false;
            goto!(1656);
        }
        1656 => {
            pins |= pins::M1 | pins::IORQ;
            goto!(1657);
        }
        1657 => {
            wait!();
            cpu.state.dlatch = gd!();
            goto!(1658);
        }
        1658 => {
            pins = cpu.refresh(pins);
            goto!(1659);
        }
        1659 => {
            goto!(1660);
        }
        1660 => {
            goto!(1661);
        }
        1661 => {
            goto!(1662);
        }
        1662 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1663);
        }
        1663 => {
            goto!(1664);
        }
        1664 => {
            goto!(1665);
        }
        1665 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.set_wz_l(cpu.state.dlatch);
            cpu.regs.set_wz_h(cpu.regs.i);
            goto!(1666);
        }
        1666 => {
            goto!(1667);
        }
        1667 => {
            goto!(1668);
        }
        1668 => {
            wait!();
            mread!(wz_postinc!());
            goto!(1669);
        }
        1669 => {
            cpu.state.dlatch = gd!();
            goto!(1670);
        }
        1670 => {
            goto!(1671);
        }
        1671 => {
            wait!();
            mread!(cpu.regs.wz);
            goto!(1672);
        }
        1672 => {
            cpu.regs.set_wz_h(gd!());
            cpu.regs.set_wz_l(cpu.state.dlatch);
            cpu.regs.pc = cpu.regs.wz;
            goto!(1673);
        }
        1673 => {
            fetch!();
        }
        1674 => {
            wait!();
            cpu.state.iff1 = false;
            goto!(1675);
        }
        1675 => {
            pins = cpu.refresh(pins);
            goto!(1676);
        }
        1676 => {
            goto!(1677);
        }
        1677 => {
            goto!(1678);
        }
        1678 => {
            goto!(1679);
        }
        1679 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_h());
            goto!(1680);
        }
        1680 => {
            goto!(1681);
        }
        1681 => {
            goto!(1682);
        }
        1682 => {
            wait!();
            mwrite!(sp_predec!(), cpu.regs.pc_l());
            cpu.regs.wz = 0x0066;
            cpu.regs.pc = cpu.regs.wz;
            goto!(1683);
        }
        1683 => {
            goto!(1684);
        }
        1684 => {
            fetch!();
        }

        crate::cpu::DDFD_M1_T2 => {
            wait!();
            cpu.state.opcode = gd!();
            goto!(crate::cpu::DDFD_M1_T3);
        }
        crate::cpu::DDFD_M1_T3 => {
            pins = cpu.refresh(pins);
            goto!(crate::cpu::DDFD_M1_T4);
        }
        crate::cpu::DDFD_M1_T4 => {
            cpu.state.addr = cpu.regs.hlx(cpu.state.hlx_idx);
            if INDIRECT_TABLE[cpu.state.opcode as usize] {
                goto!(crate::cpu::DDFD_D_T1);
            } else {
                goto!(cpu.state.opcode as u16);
            }
        }

        crate::cpu::DDFD_D_T1 => {
            goto!(crate::cpu::DDFD_D_T2);
        }
        crate::cpu::DDFD_D_T2 => {
            wait!();
            mread!(pc_postinc!());
            goto!(crate::cpu::DDFD_D_T3);
        }
        crate::cpu::DDFD_D_T3 => {
            cpu.state.addr = cpu.state.addr.wrapping_add(gd!() as i8 as u16);
            cpu.regs.wz = cpu.state.addr;
            goto!(crate::cpu::DDFD_D_T4);
        }

        crate::cpu::DDFD_D_T4 => {
            goto!(crate::cpu::DDFD_D_T5);
        }
        crate::cpu::DDFD_D_T5 => {
            if cpu.state.opcode == 0x36 {
                wait!();
                mread!(pc_postinc!());
            }
            goto!(crate::cpu::DDFD_D_T6);
        }
        crate::cpu::DDFD_D_T6 => {
            if cpu.state.opcode == 0x36 {
                cpu.state.dlatch = gd!();
            }
            goto!(crate::cpu::DDFD_D_T7);
        }
        crate::cpu::DDFD_D_T7 => {
            goto!(crate::cpu::DDFD_D_T8);
        }
        crate::cpu::DDFD_D_T8 => {
            if cpu.state.opcode == 0x36 {
                goto!(crate::cpu::DDFD_LDHLN_WR_T1);
            } else {
                goto!(cpu.state.opcode as u16);
            }
        }

        crate::cpu::DDFD_LDHLN_WR_T1 => {
            goto!(crate::cpu::DDFD_LDHLN_WR_T2);
        }
        crate::cpu::DDFD_LDHLN_WR_T2 => {
            wait!();
            mwrite!(cpu.state.addr, cpu.state.dlatch);
            goto!(crate::cpu::DDFD_LDHLN_WR_T3);
        }
        crate::cpu::DDFD_LDHLN_WR_T3 => {
            goto!(crate::cpu::DDFD_LDHLN_OVERLAPPED);
        }
        crate::cpu::DDFD_LDHLN_OVERLAPPED => {
            fetch!();
        }

        crate::cpu::CB_M1_T2 => {
            wait!();
            cpu.state.opcode = gd!();
            goto!(crate::cpu::CB_M1_T3);
        }
        crate::cpu::CB_M1_T3 => {
            pins = cpu.refresh(pins);
            goto!(crate::cpu::CB_M1_T4);
        }
        crate::cpu::CB_M1_T4 => {
            if (cpu.state.opcode & 7) == 6 {
                cpu.state.addr = cpu.regs.hl();
                goto!(crate::cpu::CBHL_STEP);
            } else {
                goto!(crate::cpu::CB_STEP);
            }
        }

        crate::cpu::ED_M1_T2 => {
            wait!();
            cpu.state.opcode = gd!();
            goto!(crate::cpu::ED_M1_T3);
        }
        crate::cpu::ED_M1_T3 => {
            pins = cpu.refresh(pins);
            goto!(crate::cpu::ED_M1_T4);
        }
        crate::cpu::ED_M1_T4 => {
            goto!((cpu.state.opcode as u16) + 256);
        }

        crate::cpu::M1_T2 => {
            wait!();
            cpu.state.opcode = gd!();
            goto!(crate::cpu::M1_T3);
        }
        crate::cpu::M1_T3 => {
            pins = cpu.refresh(pins);
            goto!(crate::cpu::M1_T4);
        }
        crate::cpu::M1_T4 => {
            cpu.state.addr = cpu.regs.hl();
            goto!(cpu.state.opcode as u16);
        }

        _ => unreachable!(),
    }
}
