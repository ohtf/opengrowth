# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""CPython repr(float) for random bit patterns, decade boundaries, and API-like values (R31)."""

import random
import struct

from _common import write

rng = random.Random(20261117)
values = set()
while len(values) < 6000:
    (x,) = struct.unpack("<d", rng.getrandbits(64).to_bytes(8, "little"))
    if x == x and abs(x) != float("inf"):
        values.add(x)
for k in range(-30, 31):
    for m in (1.0, 1.5, 9.999999999999999, 2.5):
        values.add(m * 10.0 ** k)
        values.add(-m * 10.0 ** k)
values |= {rng.uniform(-50, 50) for _ in range(2000)}
values |= {round(rng.uniform(0, 100), 4) for _ in range(2000)}
values |= {5e-324, 2.2250738585072014e-308, 1.7976931348623157e308, 0.0, -0.0, 115.0, 1e-05}
cases = [{"bits": struct.pack("<d", x).hex(), "repr": repr(x)} for x in sorted(values, key=lambda v: struct.pack("<d", v))]
write("python-repr.json", "python_repr.py", "CPython repr(float). bits is the little-endian IEEE 754 encoding in hex.", cases)
