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

//! The U880 pin bus.
//!
//! All CPU state that crosses the chip boundary is packed into a single `u64`
//! "pin mask". [`U880::tick`](crate::Cpu::tick) returns the pins the CPU is
//! driving; the host inspects them, services any memory or I/O request, and
//! passes the (possibly modified) mask back on the next tick.
//!
//! Bits 0..=15 are the address bus, bits 16..=23 the data bus, and the
//! remaining bits are individual control/status lines.

/// Address bus bit 0.
pub const A0: u64 = 1 << 0;
/// Address bus bit 1.
pub const A1: u64 = 1 << 1;
/// Address bus bit 2.
pub const A2: u64 = 1 << 2;
/// Address bus bit 3.
pub const A3: u64 = 1 << 3;
/// Address bus bit 4.
pub const A4: u64 = 1 << 4;
/// Address bus bit 5.
pub const A5: u64 = 1 << 5;
/// Address bus bit 6.
pub const A6: u64 = 1 << 6;
/// Address bus bit 7.
pub const A7: u64 = 1 << 7;
/// Address bus bit 8.
pub const A8: u64 = 1 << 8;
/// Address bus bit 9.
pub const A9: u64 = 1 << 9;
/// Address bus bit 10.
pub const A10: u64 = 1 << 10;
/// Address bus bit 11.
pub const A11: u64 = 1 << 11;
/// Address bus bit 12.
pub const A12: u64 = 1 << 12;
/// Address bus bit 13.
pub const A13: u64 = 1 << 13;
/// Address bus bit 14.
pub const A14: u64 = 1 << 14;
/// Address bus bit 15.
pub const A15: u64 = 1 << 15;

/// Data bus bit 0.
pub const D0: u64 = 1 << 16;
/// Data bus bit 1.
pub const D1: u64 = 1 << 17;
/// Data bus bit 2.
pub const D2: u64 = 1 << 18;
/// Data bus bit 3.
pub const D3: u64 = 1 << 19;
/// Data bus bit 4.
pub const D4: u64 = 1 << 20;
/// Data bus bit 5.
pub const D5: u64 = 1 << 21;
/// Data bus bit 6.
pub const D6: u64 = 1 << 22;
/// Data bus bit 7.
pub const D7: u64 = 1 << 23;

/// Machine cycle one (opcode fetch).
pub const M1: u64 = 1 << 24;
/// Memory request.
pub const MREQ: u64 = 1 << 25;
/// I/O request.
pub const IORQ: u64 = 1 << 26;
/// Read.
pub const RD: u64 = 1 << 27;
/// Write.
pub const WR: u64 = 1 << 28;
/// Halt state.
pub const HALT: u64 = 1 << 29;
/// Maskable interrupt request (set by the host).
pub const INT: u64 = 1 << 30;
/// Reset request (not emulated; call [`U880::reset`](crate::Cpu::reset)).
pub const RES: u64 = 1 << 31;
/// Non-maskable interrupt request (set by the host).
pub const NMI: u64 = 1 << 32;
/// Wait request (set by the host to inject wait states).
pub const WAIT: u64 = 1 << 33;
/// Refresh.
pub const RFSH: u64 = 1 << 34;

/// Virtual pin: unified interrupt-daisy-chain "Interrupt Enable In+Out".
pub const IEIO: u64 = 1 << 37;
/// Virtual pin: the CPU has decoded a `RETI` instruction.
pub const RETI: u64 = 1 << 38;

/// Mask of all control pins the CPU drives during a machine cycle.
pub const CTRL_MASK: u64 = M1 | MREQ | IORQ | RD | WR | RFSH;
/// Mask of every meaningful pin bit (the lowest 40 bits).
pub const PIN_MASK: u64 = (1 << 40) - 1;

/// Extracts the 16-bit address bus.
#[inline(always)]
pub fn addr(pins: u64) -> u16 {
    pins as u16
}

/// Extracts the 8-bit data bus.
#[inline(always)]
pub fn data(pins: u64) -> u8 {
    (pins >> 16) as u8
}

/// Replaces the address bus, leaving all other pins unchanged.
#[inline(always)]
pub fn set_addr(pins: u64, addr: u16) -> u64 {
    (pins & !0xFFFF) | (addr as u64)
}

/// Replaces the data bus, leaving all other pins unchanged.
///
/// This is what a host calls to put a byte onto the bus in response to a read
/// request (memory read, I/O read, or interrupt-acknowledge vector fetch).
#[inline(always)]
pub fn set_data(pins: u64, data: u8) -> u64 {
    (pins & !0xFF0000) | ((data as u64) << 16)
}

/// Replaces the address bus and ORs in the given control pins.
#[inline(always)]
pub fn set_addr_ctrl(pins: u64, addr: u16, ctrl: u64) -> u64 {
    (pins & !0xFFFF) | (addr as u64) | ctrl
}

/// Replaces the address and data buses and ORs in the given control pins.
#[inline(always)]
pub fn set_addr_data_ctrl(pins: u64, addr: u16, data: u8, ctrl: u64) -> u64 {
    (pins & !0xFFFFFF) | ((data as u64) << 16) | (addr as u64) | ctrl
}

/// Builds a fresh pin mask from control pins, an address and a data byte.
#[inline(always)]
pub fn make_pins(ctrl: u64, addr: u16, data: u8) -> u64 {
    ctrl | ((data as u64) << 16) | (addr as u64)
}
