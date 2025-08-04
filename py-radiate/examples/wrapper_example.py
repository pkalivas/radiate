#!/usr/bin/env python3
"""
this is more of a quick testing file for me
"""

import radiate as rd


codec = rd.FloatCodec([0.0, 1.0, 2.0, 3.0, 4.0, 5.0])

print(codec.encode())
