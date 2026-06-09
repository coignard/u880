# Changelog

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
