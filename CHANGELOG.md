# Changelog

## 0.1.4

### Fixed
- `WAIT` is now sampled on the data T-state of every non-`M1` memory machine cycle instead of on the address T-state, so an asserted `WAIT` pin actually stalls reads and writes the way it already did for the `M1` opcode fetch.

## 0.1.3

### Fixed
- Looping block instructions (`LDIR`, `LDDR`, `CPIR`, `CPDR`, `INIR`, `INDR`, `OTIR`, `OTDR`) now correctly leak bits 3/5 of `PC_H` into `Flags::X`/`Flags::Y` when decrementing PC, matching the U880 ALU hardware behavior

## 0.1.2

### Added

- `Revision` enum with `Older` and `Newer` variants selecting SCF/CCF undocumented flag behavior
- `Cpu::revision` field and `Cpu::with_revision(revision: Revision)` constructor
- `alu::scf` and `alu::ccf` now apply revision-dependent `Flags::X`/`Flags::Y` math: `Older` copies flags from A, `Newer` ORs A with the previous flags (`A | F_in`), matching the MME U880 hardware bug

## 0.1.1

- Added z80type test
- Renamed shadow register fields: `a_prime`, `b_prime`, etc. to `a2`, `b2`, etc.
- Updated docs
- Fixed `\r` handling in test output (zexall and z80type)

## 0.1.0

### Added

- Initial commit
