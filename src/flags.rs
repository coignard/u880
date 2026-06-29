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

//! U880 status flags register and flag-bit constants.

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
/// U880 status flags packed into a single byte.
pub struct Flags(pub u8);

impl Flags {
    /// Carry.
    pub const C: u8 = 1 << 0;
    /// Add/Subtract (set after subtractions).
    pub const N: u8 = 1 << 1;
    /// Parity (alias of the overflow bit).
    pub const P: u8 = 1 << 2;
    /// Overflow (alias of the parity bit).
    pub const V: u8 = 1 << 2;
    /// Undocumented bit 3 (copy of result bit 3).
    pub const X: u8 = 1 << 3;
    /// Half-carry.
    pub const H: u8 = 1 << 4;
    /// Undocumented bit 5 (copy of result bit 5).
    pub const Y: u8 = 1 << 5;
    /// Zero.
    pub const Z: u8 = 1 << 6;
    /// Sign.
    pub const S: u8 = 1 << 7;

    /// Returns a `Flags` value with no flags set.
    #[inline(always)]
    pub fn empty() -> Self {
        Self(0)
    }

    /// Returns `true` if all bits in `flag` are set.
    #[inline(always)]
    pub fn contains(&self, flag: u8) -> bool {
        (self.0 & flag) == flag
    }

    /// Sets all bits in `flag`.
    #[inline(always)]
    pub fn insert(&mut self, flag: u8) {
        self.0 |= flag;
    }

    /// Clears all bits in `flag`.
    #[inline(always)]
    pub fn remove(&mut self, flag: u8) {
        self.0 &= !flag;
    }

    /// Sets or clears all bits in `flag` according to `value`.
    #[inline(always)]
    pub fn set(&mut self, flag: u8, value: bool) {
        if value {
            self.insert(flag);
        } else {
            self.remove(flag);
        }
    }

    /// Returns the raw flag byte.
    #[inline(always)]
    pub fn bits(&self) -> u8 {
        self.0
    }

    /// Constructs a `Flags` value from a raw byte, preserving all bits.
    #[inline(always)]
    pub fn from_bits_retain(bits: u8) -> Self {
        Self(bits)
    }
}

impl core::ops::BitOr<u8> for Flags {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: u8) -> Self::Output {
        Self(self.0 | rhs)
    }
}
