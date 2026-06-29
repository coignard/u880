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

//! CPU execution state: decoder step, interrupt mode, and IFF flip-flops.

/// The U880 maskable-interrupt response mode, selected by the `IM 0/1/2`
/// instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum InterruptMode {
    /// Mode 0: execute an instruction placed on the data bus by the device.
    #[default]
    IM0,
    /// Mode 1: restart to `0x0038`.
    IM1,
    /// Mode 2: vectored interrupt through the table pointed to by register `I`.
    IM2,
}

/// Internal execution and interrupt state of the CPU.
///
/// Most fields are decoder bookkeeping and rarely useful to inspect; the
/// interrupt-related fields ([`im`](State::im), [`iff1`](State::iff1),
/// [`iff2`](State::iff2)) are the ones a host is most likely to read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    /// Currently active decoder step.
    pub step: u16,
    /// Effective address for `(HL)`, `(IX+d)`, `(IY+d)` accesses.
    pub addr: u16,
    /// Temporary latch for the data-bus value.
    pub dlatch: u8,
    /// Opcode currently being executed.
    pub opcode: u8,
    /// Which of HL/IX/IY the `(HL)`-style operand maps to (0: HL, 1: IX, 2: IY).
    pub hlx_idx: u8,
    /// Whether any instruction prefix is currently active.
    pub prefix_active: bool,
    /// Last pin mask, retained for NMI edge detection.
    pub pins: u64,
    /// Pending `INT`/`NMI` request tracking.
    pub int_bits: u64,
    /// Active interrupt mode.
    pub im: InterruptMode,
    /// Interrupt flip-flop 1 (master interrupt enable).
    pub iff1: bool,
    /// Interrupt flip-flop 2 (saved copy of `iff1`).
    pub iff2: bool,
}
