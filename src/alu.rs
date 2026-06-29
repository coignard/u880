// This file is part of u880.
//
// Copyright (c) 2026  René Coignard <contact@renecoignard.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::cpu::{Cpu, Revision};
use crate::flags::Flags;

#[inline(always)]
pub fn szp_flags(val: u8) -> Flags {
    let mut f = Flags::empty();
    if val == 0 {
        f.insert(Flags::Z);
    }
    if (val & 0x80) != 0 {
        f.insert(Flags::S);
    }
    if val.count_ones().is_multiple_of(2) {
        f.insert(Flags::P);
    }
    f.0 |= val & (Flags::Y | Flags::X);
    f
}

#[inline(always)]
pub fn add8(a: u8, b: u8, carry: bool) -> (u8, Flags) {
    let c = carry as u16;
    let res32 = (a as u32) + (b as u32) + (c as u32);
    let res = res32 as u8;
    let mut f = Flags::empty();
    f.0 |= res & (Flags::Y | Flags::X);
    if res & 0x80 != 0 {
        f.insert(Flags::S);
    }
    if res == 0 {
        f.insert(Flags::Z);
    }
    if (res32 & 0x100) != 0 {
        f.insert(Flags::C);
    }
    if ((a ^ b ^ res) & 0x10) != 0 {
        f.insert(Flags::H);
    }
    if ((a ^ !b) & (a ^ res) & 0x80) != 0 {
        f.insert(Flags::V);
    }
    (res, f)
}

#[inline(always)]
pub fn sub8(a: u8, b: u8, carry: bool) -> (u8, Flags) {
    let c = carry as u16;
    let res32 = (a as u32).wrapping_sub(b as u32).wrapping_sub(c as u32);
    let res = res32 as u8;
    let mut f = Flags::from_bits_retain(Flags::N);
    f.0 |= res & (Flags::Y | Flags::X);
    if res & 0x80 != 0 {
        f.insert(Flags::S);
    }
    if res == 0 {
        f.insert(Flags::Z);
    }
    if (res32 & 0x100) != 0 {
        f.insert(Flags::C);
    }
    if ((a ^ b ^ res) & 0x10) != 0 {
        f.insert(Flags::H);
    }
    if ((a ^ b) & (a ^ res) & 0x80) != 0 {
        f.insert(Flags::V);
    }
    (res, f)
}

#[inline(always)]
pub fn neg8(a: u8) -> (u8, Flags) {
    sub8(0, a, false)
}

#[inline(always)]
pub fn and8(a: u8, b: u8) -> (u8, Flags) {
    let res = a & b;
    let mut f = Flags::from_bits_retain(Flags::H);
    f.0 |= res & (Flags::Y | Flags::X);
    if res & 0x80 != 0 {
        f.insert(Flags::S);
    }
    if res == 0 {
        f.insert(Flags::Z);
    }
    if res.count_ones().is_multiple_of(2) {
        f.insert(Flags::P);
    }
    (res, f)
}

#[inline(always)]
pub fn or8(a: u8, b: u8) -> (u8, Flags) {
    let res = a | b;
    let mut f = Flags::empty();
    f.0 |= res & (Flags::Y | Flags::X);
    if res & 0x80 != 0 {
        f.insert(Flags::S);
    }
    if res == 0 {
        f.insert(Flags::Z);
    }
    if res.count_ones().is_multiple_of(2) {
        f.insert(Flags::P);
    }
    (res, f)
}

#[inline(always)]
pub fn xor8(a: u8, b: u8) -> (u8, Flags) {
    let res = a ^ b;
    let mut f = Flags::empty();
    f.0 |= res & (Flags::Y | Flags::X);
    if res & 0x80 != 0 {
        f.insert(Flags::S);
    }
    if res == 0 {
        f.insert(Flags::Z);
    }
    if res.count_ones().is_multiple_of(2) {
        f.insert(Flags::P);
    }
    (res, f)
}

#[inline(always)]
pub fn inc8(a: u8, f_in: Flags) -> (u8, Flags) {
    let res = a.wrapping_add(1);
    let mut f = Flags::from_bits_retain(f_in.0 & Flags::C);
    f.0 |= res & (Flags::Y | Flags::X);
    if res & 0x80 != 0 {
        f.insert(Flags::S);
    }
    if res == 0 {
        f.insert(Flags::Z);
    }
    if (a & 0x0F) == 0x0F {
        f.insert(Flags::H);
    }
    if a == 0x7F {
        f.insert(Flags::V);
    }
    (res, f)
}

#[inline(always)]
pub fn dec8(a: u8, f_in: Flags) -> (u8, Flags) {
    let res = a.wrapping_sub(1);
    let mut f = Flags::from_bits_retain((f_in.0 & Flags::C) | Flags::N);
    f.0 |= res & (Flags::Y | Flags::X);
    if res & 0x80 != 0 {
        f.insert(Flags::S);
    }
    if res == 0 {
        f.insert(Flags::Z);
    }
    if (a & 0x0F) == 0x00 {
        f.insert(Flags::H);
    }
    if a == 0x80 {
        f.insert(Flags::V);
    }
    (res, f)
}

#[inline(always)]
pub fn cp8(a: u8, b: u8) -> Flags {
    let (_, f) = sub8(a, b, false);
    let mut final_f = Flags::from_bits_retain(f.0 & !(Flags::Y | Flags::X));
    final_f.0 |= b & (Flags::Y | Flags::X);
    final_f
}

#[inline(always)]
pub fn daa(a: u8, f_in: Flags) -> (u8, Flags) {
    let mut res = a;
    let mut f = f_in;
    let mut diff = 0;
    let carry = f.contains(Flags::C);
    let half_carry = f.contains(Flags::H);
    let neg = f.contains(Flags::N);

    if (a & 0x0F) > 0x09 || half_carry {
        diff += 0x06;
    }
    if a > 0x99 || carry {
        diff += 0x60;
    }

    if neg {
        res = res.wrapping_sub(diff);
    } else {
        res = res.wrapping_add(diff);
    }

    f.remove(Flags::C | Flags::H | Flags::Y | Flags::X | Flags::S | Flags::Z | Flags::P);
    if carry || a > 0x99 {
        f.insert(Flags::C);
    }
    if (a ^ res) & 0x10 != 0 {
        f.insert(Flags::H);
    }
    f.0 |= res & (Flags::Y | Flags::X);
    if res & 0x80 != 0 {
        f.insert(Flags::S);
    }
    if res == 0 {
        f.insert(Flags::Z);
    }
    if res.count_ones().is_multiple_of(2) {
        f.insert(Flags::P);
    }
    (res, f)
}

#[inline(always)]
pub fn cpl(a: u8, f_in: Flags) -> (u8, Flags) {
    let res = !a;
    let mut f = Flags::from_bits_retain(f_in.0 & (Flags::S | Flags::Z | Flags::P | Flags::C));
    f.insert(Flags::H | Flags::N);
    f.0 |= res & (Flags::Y | Flags::X);
    (res, f)
}

#[inline(always)]
pub fn scf(a: u8, f_in: Flags, rev: Revision) -> Flags {
    let mut f = Flags::from_bits_retain(f_in.0 & (Flags::S | Flags::Z | Flags::P | Flags::C));
    f.insert(Flags::C);
    match rev {
        Revision::Older => f.0 |= a & (Flags::Y | Flags::X),
        Revision::Newer => f.0 |= (a | f_in.0) & (Flags::Y | Flags::X),
    }
    f
}

#[inline(always)]
pub fn ccf(a: u8, f_in: Flags, rev: Revision) -> Flags {
    let mut f = Flags::from_bits_retain(f_in.0 & (Flags::S | Flags::Z | Flags::P | Flags::C));
    let carry = if f_in.contains(Flags::C) { 1 } else { 0 };
    f.0 |= carry << 4;
    match rev {
        Revision::Older => f.0 |= a & (Flags::Y | Flags::X),
        Revision::Newer => f.0 |= (a | f_in.0) & (Flags::Y | Flags::X),
    }
    if carry == 0 {
        f.insert(Flags::C);
    } else {
        f.remove(Flags::C);
    }
    f
}

#[inline(always)]
pub fn add16(cpu: &mut Cpu, val: u16) {
    let acc = cpu.regs.hlx(cpu.state.hlx_idx);
    cpu.regs.wz = acc.wrapping_add(1);
    let res32 = (acc as u32) + (val as u32);
    let res = res32 as u16;
    cpu.regs.set_hlx(cpu.state.hlx_idx, res);

    let mut f = Flags::from_bits_retain(cpu.regs.f.bits() & (Flags::S | Flags::Z | Flags::V));
    f.0 |= (((acc ^ res ^ val) >> 8) as u8) & Flags::H;
    if (res32 & 0x1_00_00) != 0 {
        f.insert(Flags::C);
    }
    f.0 |= (res >> 8) as u8 & (Flags::Y | Flags::X);
    cpu.regs.f = f;
}

#[inline(always)]
pub fn adc16(cpu: &mut Cpu, val: u16) {
    let acc = cpu.regs.hl();
    cpu.regs.wz = acc.wrapping_add(1);
    let c = if cpu.regs.f.contains(Flags::C) { 1 } else { 0 };
    let res32 = (acc as u32) + (val as u32) + c;
    let res = res32 as u16;
    cpu.regs.set_hl(res);

    let mut f = Flags::empty();
    let v_flag = ((val ^ acc ^ 0x8000) & (val ^ res) & 0x8000) >> 13;
    f.0 |= v_flag as u8;
    f.0 |= (((acc ^ res ^ val) >> 8) as u8) & Flags::H;
    if (res32 & 0x1_00_00) != 0 {
        f.insert(Flags::C);
    }
    f.0 |= (res >> 8) as u8 & (Flags::S | Flags::Y | Flags::X);
    if res == 0 {
        f.insert(Flags::Z);
    }
    cpu.regs.f = f;
}

#[inline(always)]
pub fn sbc16(cpu: &mut Cpu, val: u16) {
    let acc = cpu.regs.hl();
    cpu.regs.wz = acc.wrapping_add(1);
    let c = if cpu.regs.f.contains(Flags::C) { 1 } else { 0 };
    let res32 = (acc as u32).wrapping_sub(val as u32).wrapping_sub(c);
    let res = res32 as u16;
    cpu.regs.set_hl(res);

    let mut f = Flags::from_bits_retain(Flags::N);
    let v_flag = ((val ^ acc) & (acc ^ res) & 0x8000) >> 13;
    f.0 |= v_flag as u8;
    f.0 |= (((acc ^ res ^ val) >> 8) as u8) & Flags::H;
    if (res32 & 0x1_00_00) != 0 {
        f.insert(Flags::C);
    }
    f.0 |= (res >> 8) as u8 & (Flags::S | Flags::Y | Flags::X);
    if res == 0 {
        f.insert(Flags::Z);
    }
    cpu.regs.f = f;
}

#[inline(always)]
pub fn rlc(val: u8) -> (u8, Flags) {
    let res = val.rotate_left(1);
    let mut f = szp_flags(res);
    if (val >> 7) != 0 {
        f.insert(Flags::C);
    }
    (res, f)
}

#[inline(always)]
pub fn rrc(val: u8) -> (u8, Flags) {
    let res = val.rotate_right(1);
    let mut f = szp_flags(res);
    if (val & 1) != 0 {
        f.insert(Flags::C);
    }
    (res, f)
}

#[inline(always)]
pub fn rl(val: u8, f_in: Flags) -> (u8, Flags) {
    let c = if f_in.contains(Flags::C) { 1 } else { 0 };
    let res = (val << 1) | c;
    let mut f = szp_flags(res);
    if (val >> 7) != 0 {
        f.insert(Flags::C);
    }
    (res, f)
}

#[inline(always)]
pub fn rr(val: u8, f_in: Flags) -> (u8, Flags) {
    let c = if f_in.contains(Flags::C) { 0x80 } else { 0 };
    let res = (val >> 1) | c;
    let mut f = szp_flags(res);
    if (val & 1) != 0 {
        f.insert(Flags::C);
    }
    (res, f)
}

#[inline(always)]
pub fn sla(val: u8) -> (u8, Flags) {
    let res = val << 1;
    let mut f = szp_flags(res);
    if (val >> 7) != 0 {
        f.insert(Flags::C);
    }
    (res, f)
}

#[inline(always)]
pub fn sra(val: u8) -> (u8, Flags) {
    let res = (val >> 1) | (val & 0x80);
    let mut f = szp_flags(res);
    if (val & 1) != 0 {
        f.insert(Flags::C);
    }
    (res, f)
}

#[inline(always)]
pub fn sll(val: u8) -> (u8, Flags) {
    let res = (val << 1) | 1;
    let mut f = szp_flags(res);
    if (val >> 7) != 0 {
        f.insert(Flags::C);
    }
    (res, f)
}

#[inline(always)]
pub fn srl(val: u8) -> (u8, Flags) {
    let res = val >> 1;
    let mut f = szp_flags(res);
    if (val & 1) != 0 {
        f.insert(Flags::C);
    }
    (res, f)
}

#[inline(always)]
pub fn bit(y: u8, val: u8, f_in: Flags, is_mem: bool, wz_high: u8) -> Flags {
    let res = val & (1 << y);
    let mut f = Flags::from_bits_retain(f_in.0 & Flags::C);
    f.insert(Flags::H);
    if res == 0 {
        f.insert(Flags::Z | Flags::P);
    } else if res & 0x80 != 0 {
        f.insert(Flags::S);
    }
    if is_mem {
        f.0 |= wz_high & (Flags::Y | Flags::X);
    } else {
        f.0 |= val & (Flags::Y | Flags::X);
    }
    f
}

#[inline(always)]
pub fn cb_action(cpu: &mut Cpu, z0: u8, z1: u8) -> bool {
    let x = cpu.state.opcode >> 6;
    let y = (cpu.state.opcode >> 3) & 7;

    let val = match z0 {
        0 => cpu.regs.b,
        1 => cpu.regs.c,
        2 => cpu.regs.d,
        3 => cpu.regs.e,
        4 => cpu.regs.h,
        5 => cpu.regs.l,
        6 => cpu.state.dlatch,
        7 => cpu.regs.a,
        _ => unreachable!(),
    };

    let res = match x {
        0 => {
            let (r, f) = match y {
                0 => rlc(val),
                1 => rrc(val),
                2 => rl(val, cpu.regs.f),
                3 => rr(val, cpu.regs.f),
                4 => sla(val),
                5 => sra(val),
                6 => sll(val),
                7 => srl(val),
                _ => unreachable!(),
            };
            cpu.regs.f = f;
            r
        }
        1 => {
            cpu.regs.f = bit(y, val, cpu.regs.f, z0 == 6, cpu.regs.wz_h());
            return false;
        }
        2 => val & !(1 << y),
        3 => val | (1 << y),
        _ => unreachable!(),
    };

    cpu.state.dlatch = res;
    match z1 {
        0 => cpu.regs.b = res,
        1 => cpu.regs.c = res,
        2 => cpu.regs.d = res,
        3 => cpu.regs.e = res,
        4 => cpu.regs.h = res,
        5 => cpu.regs.l = res,
        6 => {}
        7 => cpu.regs.a = res,
        _ => unreachable!(),
    }
    true
}

#[inline(always)]
pub fn ldi_ldd_flags(cpu: &mut Cpu, val: u8) -> bool {
    let res = cpu.regs.a.wrapping_add(val);
    let bc = cpu.regs.bc().wrapping_sub(1);
    cpu.regs.set_bc(bc);

    let mut f = Flags::from_bits_retain(cpu.regs.f.bits() & (Flags::S | Flags::Z | Flags::C));
    if (res & 2) != 0 {
        f.insert(Flags::Y);
    }
    if (res & 8) != 0 {
        f.insert(Flags::X);
    }
    if bc != 0 {
        f.insert(Flags::V);
    }
    cpu.regs.f = f;

    bc != 0
}

#[inline(always)]
pub fn cpi_cpd_flags(cpu: &mut Cpu, val: u8) -> bool {
    let mut res = cpu.regs.a.wrapping_sub(val);
    let bc = cpu.regs.bc().wrapping_sub(1);
    cpu.regs.set_bc(bc);

    let mut f = Flags::from_bits_retain((cpu.regs.f.bits() & Flags::C) | Flags::N);
    if res == 0 {
        f.insert(Flags::Z);
    }
    if (res & 0x80) != 0 {
        f.insert(Flags::S);
    }

    if (cpu.regs.a & 0x0F) < (val & 0x0F) {
        f.insert(Flags::H);
        res = res.wrapping_sub(1);
    }

    if (res & 2) != 0 {
        f.insert(Flags::Y);
    }
    if (res & 8) != 0 {
        f.insert(Flags::X);
    }
    if bc != 0 {
        f.insert(Flags::V);
    }
    cpu.regs.f = f;

    bc != 0 && !f.contains(Flags::Z)
}

#[inline(always)]
pub fn ini_ind_flags(cpu: &mut Cpu, val: u8, c: u8) -> bool {
    let b = cpu.regs.b;
    let mut f =
        Flags::from_bits_retain((szp_flags(b).bits() & !Flags::P) | (cpu.regs.f.bits() & Flags::C));

    if val & 0x80 != 0 {
        f.insert(Flags::N);
    }

    let t = (c as u16) + (val as u16);
    if t > 0xFF {
        f.insert(Flags::H);
    }

    let p_val = ((t & 7) as u8) ^ b;
    if p_val.count_ones().is_multiple_of(2) {
        f.insert(Flags::P);
    }

    cpu.regs.f = f;
    b != 0
}

#[inline(always)]
pub fn outi_outd_flags(cpu: &mut Cpu, val: u8) -> bool {
    let b = cpu.regs.b;
    let mut f =
        Flags::from_bits_retain((szp_flags(b).bits() & !Flags::P) | (cpu.regs.f.bits() & Flags::C));

    if val & 0x80 != 0 {
        f.insert(Flags::N);
    }

    let t = (cpu.regs.l as u16) + (val as u16);
    if t > 0xFF {
        f.insert(Flags::H);
    }

    let p_val = ((t & 7) as u8) ^ b;
    if p_val.count_ones().is_multiple_of(2) {
        f.insert(Flags::P);
    }

    cpu.regs.f = f;
    b != 0
}

#[inline(always)]
pub fn rrd(cpu: &mut Cpu, val: u8) -> u8 {
    let l = cpu.regs.a & 0x0F;
    cpu.regs.a = (cpu.regs.a & 0xF0) | (val & 0x0F);
    let res = (val >> 4) | (l << 4);
    cpu.regs.f.0 = (cpu.regs.f.bits() & Flags::C) | szp_flags(cpu.regs.a).bits();
    res
}

#[inline(always)]
pub fn rld(cpu: &mut Cpu, val: u8) -> u8 {
    let l = cpu.regs.a & 0x0F;
    cpu.regs.a = (cpu.regs.a & 0xF0) | (val >> 4);
    let res = (val << 4) | l;
    cpu.regs.f.0 = (cpu.regs.f.bits() & Flags::C) | szp_flags(cpu.regs.a).bits();
    res
}

#[inline(always)]
pub fn sziff2_flags(cpu: &Cpu, val: u8) -> u8 {
    let mut f = (cpu.regs.f.bits() & Flags::C) | (val & (Flags::Y | Flags::X));
    if val == 0 {
        f |= Flags::Z;
    } else if (val & 0x80) != 0 {
        f |= Flags::S;
    }
    if cpu.state.iff2 {
        f |= Flags::P;
    }
    f
}
