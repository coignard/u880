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

//! U880 registers: main and shadow banks, index registers, and special-purpose registers.

use crate::flags::Flags;

/// U880 registers.
///
/// Holds all programmer-visible and internal registers: the main and shadow
/// banks (AF, BC, DE, HL and their alternates), index registers IX and IY,
/// stack pointer SP, program counter PC, interrupt vector base I, memory
/// refresh counter R, and the internal WZ (MEMPTR) register.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    /// Accumulator.
    pub a: u8,
    /// Flags register.
    pub f: Flags,
    /// General-purpose register B.
    pub b: u8,
    /// General-purpose register C.
    pub c: u8,
    /// General-purpose register D.
    pub d: u8,
    /// General-purpose register E.
    pub e: u8,
    /// General-purpose register H.
    pub h: u8,
    /// General-purpose register L.
    pub l: u8,

    /// Shadow accumulator (AF').
    pub a2: u8,
    /// Shadow flags register (AF').
    pub f2: Flags,
    /// Shadow register B'.
    pub b2: u8,
    /// Shadow register C'.
    pub c2: u8,
    /// Shadow register D'.
    pub d2: u8,
    /// Shadow register E'.
    pub e2: u8,
    /// Shadow register H'.
    pub h2: u8,
    /// Shadow register L'.
    pub l2: u8,

    /// Index register IX.
    pub ix: u16,
    /// Index register IY.
    pub iy: u16,
    /// Stack pointer.
    pub sp: u16,
    /// Program counter.
    pub pc: u16,

    /// Interrupt vector base register.
    pub i: u8,
    /// Memory refresh counter.
    pub r: u8,

    /// Internal WZ register (MEMPTR), used by many instructions to track addresses for flag computation.
    pub wz: u16,
}

impl Registers {
    /// Returns AF as a 16-bit value with A in the high byte.
    #[inline(always)]
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f.bits() as u16)
    }

    /// Loads AF from a 16-bit value; A receives the high byte, F the low byte.
    #[inline(always)]
    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = Flags::from_bits_retain(val as u8);
    }

    /// Returns BC as a 16-bit value with B in the high byte.
    #[inline(always)]
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    /// Loads BC from a 16-bit value; B receives the high byte, C the low byte.
    #[inline(always)]
    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = val as u8;
    }

    /// Returns DE as a 16-bit value with D in the high byte.
    #[inline(always)]
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    /// Loads DE from a 16-bit value; D receives the high byte, E the low byte.
    #[inline(always)]
    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = val as u8;
    }

    /// Returns HL as a 16-bit value with H in the high byte.
    #[inline(always)]
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    /// Loads HL from a 16-bit value; H receives the high byte, L the low byte.
    #[inline(always)]
    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = val as u8;
    }

    /// Returns the HL/IX/IY register selected by `idx` (0: HL, 1: IX, 2: IY).
    #[inline(always)]
    pub fn hlx(&self, idx: u8) -> u16 {
        match idx {
            0 => self.hl(),
            1 => self.ix,
            _ => self.iy,
        }
    }

    /// Writes `val` into the register selected by `idx` (0: HL, 1: IX, 2: IY).
    #[inline(always)]
    pub fn set_hlx(&mut self, idx: u8, val: u16) {
        match idx {
            0 => self.set_hl(val),
            1 => self.ix = val,
            _ => self.iy = val,
        }
    }

    /// Returns the high byte of the register selected by `idx`.
    #[inline(always)]
    pub fn hlx_h(&self, idx: u8) -> u8 {
        (self.hlx(idx) >> 8) as u8
    }

    /// Returns the low byte of the register selected by `idx`.
    #[inline(always)]
    pub fn hlx_l(&self, idx: u8) -> u8 {
        self.hlx(idx) as u8
    }

    /// Replaces the high byte of the register selected by `idx`, preserving the low byte.
    #[inline(always)]
    pub fn set_hlx_h(&mut self, idx: u8, val: u8) {
        let v = (self.hlx(idx) & 0x00FF) | ((val as u16) << 8);
        self.set_hlx(idx, v);
    }

    /// Replaces the low byte of the register selected by `idx`, preserving the high byte.
    #[inline(always)]
    pub fn set_hlx_l(&mut self, idx: u8, val: u8) {
        let v = (self.hlx(idx) & 0xFF00) | (val as u16);
        self.set_hlx(idx, v);
    }

    /// Returns the low byte of the WZ (MEMPTR) register.
    #[inline(always)]
    pub fn wz_l(&self) -> u8 {
        self.wz as u8
    }

    /// Returns the high byte of the WZ (MEMPTR) register.
    #[inline(always)]
    pub fn wz_h(&self) -> u8 {
        (self.wz >> 8) as u8
    }

    /// Replaces the low byte of WZ, preserving the high byte.
    #[inline(always)]
    pub fn set_wz_l(&mut self, val: u8) {
        self.wz = (self.wz & 0xFF00) | (val as u16);
    }

    /// Replaces the high byte of WZ, preserving the low byte.
    #[inline(always)]
    pub fn set_wz_h(&mut self, val: u8) {
        self.wz = (self.wz & 0x00FF) | ((val as u16) << 8);
    }

    /// Returns the low byte of SP.
    #[inline(always)]
    pub fn sp_l(&self) -> u8 {
        self.sp as u8
    }

    /// Returns the high byte of SP.
    #[inline(always)]
    pub fn sp_h(&self) -> u8 {
        (self.sp >> 8) as u8
    }

    /// Replaces the low byte of SP, preserving the high byte.
    #[inline(always)]
    pub fn set_sp_l(&mut self, val: u8) {
        self.sp = (self.sp & 0xFF00) | (val as u16);
    }

    /// Replaces the high byte of SP, preserving the low byte.
    #[inline(always)]
    pub fn set_sp_h(&mut self, val: u8) {
        self.sp = (self.sp & 0x00FF) | ((val as u16) << 8);
    }

    /// Returns the low byte of PC.
    #[inline(always)]
    pub fn pc_l(&self) -> u8 {
        self.pc as u8
    }

    /// Returns the high byte of PC.
    #[inline(always)]
    pub fn pc_h(&self) -> u8 {
        (self.pc >> 8) as u8
    }

    /// Replaces the low byte of PC, preserving the high byte.
    #[inline(always)]
    pub fn set_pc_l(&mut self, val: u8) {
        self.pc = (self.pc & 0xFF00) | (val as u16);
    }

    /// Replaces the high byte of PC, preserving the low byte.
    #[inline(always)]
    pub fn set_pc_h(&mut self, val: u8) {
        self.pc = (self.pc & 0x00FF) | ((val as u16) << 8);
    }

    /// Exchanges AF with the shadow register AF'.
    #[inline(always)]
    pub fn swap_af(&mut self) {
        core::mem::swap(&mut self.a, &mut self.a2);
        core::mem::swap(&mut self.f, &mut self.f2);
    }

    /// Exchanges BC, DE, and HL with their shadow counterparts (EXX).
    #[inline(always)]
    pub fn swap_exx(&mut self) {
        core::mem::swap(&mut self.b, &mut self.b2);
        core::mem::swap(&mut self.c, &mut self.c2);
        core::mem::swap(&mut self.d, &mut self.d2);
        core::mem::swap(&mut self.e, &mut self.e2);
        core::mem::swap(&mut self.h, &mut self.h2);
        core::mem::swap(&mut self.l, &mut self.l2);
    }
}
