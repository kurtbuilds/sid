import sys

import numpy

alphabet = "0123456789abcdefghjkmnpqrstvwxyz"

orig = numpy.array([ord(a) for a in alphabet])

size = 38
reduce = orig % size

def count(alphabet):
    s = set(alphabet)
    s = s - set([255])
    return len(set(s))

vals = numpy.array([255 for _ in range(size)])
vals[reduce] = numpy.array(range(0, 32))
if count(vals) != 32:
    print('Something went wrong')
    sys.exit(1)
print(repr(vals))

print(', '.join(f"'{c}'" for c in alphabet))