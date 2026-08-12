#!/usr/bin/env python3
"""Create verified, game-local patches; originals are retained beside them."""
import struct
import sys
from pathlib import Path


def game_root_from_args() -> Path:
    if len(sys.argv) != 2:
        raise SystemExit(f"usage: {sys.argv[0]} game-directory")
    return Path(sys.argv[1]).expanduser().resolve()


WIN10 = game_root_from_args() / "bin" / "win10"


def rva_offset(image: bytes, rva: int) -> int:
    pe = struct.unpack_from("<I", image, 0x3C)[0]
    count = struct.unpack_from("<H", image, pe + 6)[0]
    optional_size = struct.unpack_from("<H", image, pe + 20)[0]
    sections = pe + 24 + optional_size
    for index in range(count):
        section = sections + index * 40
        virtual_size, virtual_address, raw_size, raw_offset = struct.unpack_from(
            "<IIII", image, section + 8
        )
        if virtual_address <= rva < virtual_address + max(virtual_size, raw_size):
            return raw_offset + rva - virtual_address
    raise ValueError(f"RVA {rva:#x} is not in a section")


def patch(image: bytearray, rva: int, expected: bytes, replacement: bytes) -> None:
    offset = rva_offset(image, rva)
    actual = bytes(image[offset : offset + len(expected)])
    if actual != expected:
        raise RuntimeError(
            f"{rva:#x}: expected {expected.hex()}, found {actual.hex()}"
        )
    image[offset : offset + len(replacement)] = replacement


def original_file(filename: str) -> Path:
    current = WIN10 / filename
    original = WIN10 / f"{filename}.original"
    if not original.exists():
        original.write_bytes(current.read_bytes())
    return original


# How these sites were determined (specific to the SHA-256-verified originals
# retained below):
#
# 1. Disassemble echovr.exe (`objdump -d -Mintel echovr.exe`) and follow the
#    local DLL-loader path. At RVA 0x1365b1c it calls the file verification
#    helper; the immediately following `test eax,eax; jne +0x11` at 0x1365b21
#    selects the existing LoadLibraryW path only when verification succeeds.
#    NOP the test and turn JNE into JMP, retaining the verifier's ABI/function
#    body while always selecting that already-existing load path.
#
# 2. The former runtime bypass had identified pnsovr's PreLoaded result branch.
#    PE section/RVA translation and a fresh disassembly verify that RVA 0x98b5a
#    contains `je +0x27`; its fall-through stores -2
#    (ovrPlatformInitialize_PreLoaded). Changing only JE to JMP takes pnsovr's
#    own existing Platform API resolution path. The expected-byte checks below
#    deliberately reject a different game/pnsovr build instead of patching it.
#
# The original files are never modified; this script reconstructs patched game
# files from them on every execution.

# Echo's file-verification helper remains intact. Only its caller's conditional
# branch is changed to follow the existing normal LoadLibraryW path.
echo_original = original_file("echovr.exe")
echo = bytearray(echo_original.read_bytes())
patch(echo, 0x1365B21, bytes.fromhex("85c07511"), bytes.fromhex("9090eb11"))
(WIN10 / "echovr.exe").write_bytes(echo)

# pnsovr's preloaded-platform failure branch changes from JE (return -2) to an
# existing success-path JMP. This is the same two-byte in-memory patch formerly
# applied by the deleted injector, now deterministic and game-local.
pns_original = original_file("pnsovr.dll")
pns = bytearray(pns_original.read_bytes())
patch(pns, 0x98B5A, bytes.fromhex("7427"), bytes.fromhex("eb27"))
(WIN10 / "pnsovr.dll").write_bytes(pns)
