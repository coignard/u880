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

//! A cycle-stepped MME U880 emulator.
//!
//! # Example
//!
//! ```
//! use u880::{Cpu, pins};
//!
//! let mut cpu = Cpu::new();
//! let mut mem = [0u8; 1 << 16];
//! let mut bus = 0u64;
//!
//! for _ in 0..32 {
//!     bus = cpu.tick(bus);
//!     if bus & pins::MREQ != 0 {
//!         let addr = pins::addr(bus);
//!         if bus & pins::RD != 0 {
//!             bus = pins::set_data(bus, mem[addr as usize]);
//!         } else if bus & pins::WR != 0 {
//!             mem[addr as usize] = pins::data(bus);
//!         }
//!     }
//! }
//! ```

#![no_std]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod cpu;
pub mod flags;
pub mod pins;
pub mod registers;
pub mod state;

mod alu;
mod decoder;

pub use cpu::{Cpu, Revision};
pub use flags::Flags;
pub use registers::Registers;
pub use state::{InterruptMode, State};
