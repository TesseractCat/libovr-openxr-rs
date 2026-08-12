#!/usr/bin/env python3
"""Create verified, game-local patches; originals are retained beside them."""
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WIN10 = ROOT / "bin" / "win10"


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


# Echo's file-verification helper remains intact. Only its caller's conditional
# branch is changed to follow the existing normal LoadLibraryW path.
echo_original = WIN10 / "echovr.exe.original"
echo = bytearray(echo_original.read_bytes())
patch(echo, 0x1365B21, bytes.fromhex("85c07511"), bytes.fromhex("9090eb11"))
(WIN10 / "echovr.exe").write_bytes(echo)

# pnsovr's preloaded-platform failure branch changes from JE (return -2) to an
# existing success-path JMP. This is the same two-byte in-memory patch formerly
# applied by the deleted injector, now deterministic and game-local.
pns_original = WIN10 / "pnsovr.dll.original"
if not pns_original.exists():
    pns_original.write_bytes((WIN10 / "pnsovr.dll").read_bytes())
pns = bytearray(pns_original.read_bytes())
patch(pns, 0x98B5A, bytes.fromhex("7427"), bytes.fromhex("eb27"))
(WIN10 / "pnsovr.dll").write_bytes(pns)
