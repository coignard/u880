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

//! The U880 CPU struct and its primary interface: [`Cpu::new`], [`Cpu::tick`], and [`Cpu::reset`].

use crate::pins;
use crate::registers::Registers;
use crate::state::{InterruptMode, State};

pub(crate) const DDFD_M1_T2: u16 = 1685;
pub(crate) const DDFD_M1_T3: u16 = 1686;
pub(crate) const DDFD_M1_T4: u16 = 1687;
pub(crate) const DDFD_D_T1: u16 = 1688;
pub(crate) const DDFD_D_T2: u16 = 1689;
pub(crate) const DDFD_D_T3: u16 = 1690;
pub(crate) const DDFD_D_T4: u16 = 1691;
pub(crate) const DDFD_D_T5: u16 = 1692;
pub(crate) const DDFD_D_T6: u16 = 1693;
pub(crate) const DDFD_D_T7: u16 = 1694;
pub(crate) const DDFD_D_T8: u16 = 1695;
pub(crate) const DDFD_LDHLN_WR_T1: u16 = 1696;
pub(crate) const DDFD_LDHLN_WR_T2: u16 = 1697;
pub(crate) const DDFD_LDHLN_WR_T3: u16 = 1698;
pub(crate) const DDFD_LDHLN_OVERLAPPED: u16 = 1699;
pub(crate) const CB_M1_T2: u16 = 1700;
pub(crate) const CB_M1_T3: u16 = 1701;
pub(crate) const CB_M1_T4: u16 = 1702;
pub(crate) const ED_M1_T2: u16 = 1703;
pub(crate) const ED_M1_T3: u16 = 1704;
pub(crate) const ED_M1_T4: u16 = 1705;
pub(crate) const M1_T2: u16 = 1706;
pub(crate) const M1_T3: u16 = 1707;
pub(crate) const M1_T4: u16 = 1708;

pub(crate) const CB_STEP: u16 = 1612;
pub(crate) const CBHL_STEP: u16 = 1613;
pub(crate) const DDFDCB_STEP: u16 = 1621;
pub(crate) const INT_IM0_STEP: u16 = 1636;
pub(crate) const INT_IM1_STEP: u16 = 1642;
pub(crate) const INT_IM2_STEP: u16 = 1655;
pub(crate) const NMI_STEP: u16 = 1674;

/// Selects the hardware revision for SCF/CCF undocumented flag behavior.
///
/// The [`Flags::X`] and [`Flags::Y`] flags are computed differently depending
/// on the U880 silicon revision during `SCF` and `CCF` instructions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Revision {
    /// Early MME U880: [`Flags::X`] and [`Flags::Y`] are copied from the accumulator (`F = A`).
    #[default]
    Older,
    /// Late MME U880: a hardware bug causes [`Flags::X`] and [`Flags::Y`] to be
    /// set as `A | F_in` instead of `A`
    Newer,
}

/// A U880 CPU.
///
/// Construct one with [`Cpu::new`] and advance it one clock cycle at a time
/// with [`Cpu::tick`]. Registers and flags are reachable through the public
/// [`regs`](Cpu::regs) field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cpu {
    /// The register file (main and shadow banks, IX/IY, SP, PC, I, R, WZ).
    pub regs: Registers,
    /// Internal execution and interrupt state.
    pub state: State,
    /// Hardware revision controlling SCF/CCF undocumented flag behavior.
    pub revision: Revision,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    /// Creates a CPU in its power-on state, ready to fetch from address `0x0000`.
    pub fn new() -> Self {
        Self::with_revision(Revision::default())
    }

    /// Creates a CPU with the specified hardware [`Revision`].
    pub fn with_revision(revision: Revision) -> Self {
        let mut cpu = Self {
            regs: Registers::default(),
            state: State::default(),
            revision,
        };
        cpu.reset();
        cpu
    }

    /// Resets the CPU to its power-on state and returns the initial pin mask.
    pub fn reset(&mut self) -> u64 {
        self.regs = Registers::default();
        self.regs.set_af(0xFFFF);
        self.regs.set_bc(0xFFFF);
        self.regs.set_de(0xFFFF);
        self.regs.set_hl(0xFFFF);
        self.regs.ix = 0xFFFF;
        self.regs.iy = 0xFFFF;
        self.regs.sp = 0xFFFF;
        self.regs.wz = 0xFFFF;

        self.regs.a2 = 0xFF;
        self.regs.f2 = crate::flags::Flags::from_bits_retain(0xFF);
        self.regs.b2 = 0xFF;
        self.regs.c2 = 0xFF;
        self.regs.d2 = 0xFF;
        self.regs.e2 = 0xFF;
        self.regs.h2 = 0xFF;
        self.regs.l2 = 0xFF;

        self.state = State::default();
        self.prefetch(0x0000)
    }

    /// Forces execution to continue at `new_pc`. Pass the returned mask to the
    /// next [`tick`](Cpu::tick).
    pub fn prefetch(&mut self, new_pc: u16) -> u64 {
        self.regs.pc = new_pc;
        self.state.step = 0;
        0
    }

    /// Returns `true` once the current instruction has finished (i.e. the next
    /// tick will begin a fresh opcode fetch).
    pub fn opdone(&self) -> bool {
        ((self.state.pins & (pins::M1 | pins::RD)) == (pins::M1 | pins::RD))
            && !self.state.prefix_active
    }

    #[inline(always)]
    pub(crate) fn refresh(&mut self, pins: u64) -> u64 {
        let ir = ((self.regs.i as u16) << 8) | (self.regs.r as u16);
        let out_pins = pins::set_addr_ctrl(pins, ir, pins::MREQ | pins::RFSH);
        self.regs.r = (self.regs.r & 0x80) | ((self.regs.r.wrapping_add(1)) & 0x7F);
        out_pins
    }

    #[inline(always)]
    pub(crate) fn fetch(&mut self, mut pins: u64) -> u64 {
        self.state.hlx_idx = 0;
        self.state.prefix_active = false;

        if self.state.int_bits == 0 {
            self.state.step = M1_T2;
            let pc = self.regs.pc;
            self.regs.pc = self.regs.pc.wrapping_add(1);
            pins::set_addr_ctrl(pins, pc, pins::M1 | pins::MREQ | pins::RD)
        } else if (self.state.int_bits & pins::NMI) != 0 {
            self.state.step = NMI_STEP;
            self.state.int_bits = 0;
            if (pins & pins::HALT) != 0 {
                pins &= !pins::HALT;
                self.regs.pc = self.regs.pc.wrapping_add(1);
            }
            pins::set_addr_ctrl(pins, self.regs.pc, pins::M1 | pins::MREQ | pins::RD)
        } else if (self.state.int_bits & pins::INT) != 0 {
            if self.state.iff1 {
                self.state.step = match self.state.im {
                    InterruptMode::IM0 => INT_IM0_STEP,
                    InterruptMode::IM1 => INT_IM1_STEP,
                    InterruptMode::IM2 => INT_IM2_STEP,
                };
                self.state.int_bits = 0;
                if (pins & pins::HALT) != 0 {
                    pins &= !pins::HALT;
                    self.regs.pc = self.regs.pc.wrapping_add(1);
                }
                pins
            } else {
                self.state.step = M1_T2;
                let pc = self.regs.pc;
                self.regs.pc = self.regs.pc.wrapping_add(1);
                pins::set_addr_ctrl(pins, pc, pins::M1 | pins::MREQ | pins::RD)
            }
        } else {
            pins
        }
    }

    #[inline(always)]
    pub(crate) fn fetch_cb(&mut self, pins: u64) -> u64 {
        self.state.prefix_active = true;
        if self.state.hlx_idx > 0 {
            self.state.step = DDFDCB_STEP;
            pins
        } else {
            self.state.step = CB_M1_T2;
            let pc = self.regs.pc;
            self.regs.pc = self.regs.pc.wrapping_add(1);
            pins::set_addr_ctrl(pins, pc, pins::M1 | pins::MREQ | pins::RD)
        }
    }

    #[inline(always)]
    pub(crate) fn fetch_dd(&mut self, pins: u64) -> u64 {
        self.state.step = DDFD_M1_T2;
        self.state.hlx_idx = 1;
        self.state.prefix_active = true;
        let pc = self.regs.pc;
        self.regs.pc = self.regs.pc.wrapping_add(1);
        pins::set_addr_ctrl(pins, pc, pins::M1 | pins::MREQ | pins::RD)
    }

    #[inline(always)]
    pub(crate) fn fetch_fd(&mut self, pins: u64) -> u64 {
        self.state.step = DDFD_M1_T2;
        self.state.hlx_idx = 2;
        self.state.prefix_active = true;
        let pc = self.regs.pc;
        self.regs.pc = self.regs.pc.wrapping_add(1);
        pins::set_addr_ctrl(pins, pc, pins::M1 | pins::MREQ | pins::RD)
    }

    #[inline(always)]
    pub(crate) fn fetch_ed(&mut self, pins: u64) -> u64 {
        self.state.step = ED_M1_T2;
        self.state.hlx_idx = 0;
        self.state.prefix_active = true;
        let pc = self.regs.pc;
        self.regs.pc = self.regs.pc.wrapping_add(1);
        pins::set_addr_ctrl(pins, pc, pins::M1 | pins::MREQ | pins::RD)
    }

    #[inline(always)]
    pub(crate) fn halt(&mut self, pins: u64) -> u64 {
        self.regs.pc = self.regs.pc.wrapping_sub(1);
        pins | pins::HALT
    }

    /// Advances the CPU by one clock cycle.
    ///
    /// Pass in the current pin mask (with any `INT`/`NMI`/`WAIT` requests the
    /// host wants to raise) and feed the returned mask back on the next call,
    /// after servicing any memory or I/O request it signals.
    pub fn tick(&mut self, mut pins: u64) -> u64 {
        pins &= !(pins::CTRL_MASK | pins::RETI);
        pins = crate::decoder::decode(self, pins);

        let rising_nmi = (pins ^ self.state.pins) & pins;
        self.state.pins = pins;
        self.state.int_bits = ((self.state.int_bits | rising_nmi) & pins::NMI) | (pins & pins::INT);

        pins
    }
}
