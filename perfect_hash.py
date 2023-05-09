import sys

import numpy

alphabet = "023456789abcdefghjkmnpqrstuvwxyz"

orig = numpy.array([ord(a) for a in alphabet])

reduce = orig % 38

def count(alphabet):
    s = set(alphabet)
    s = s - set([255])
    return len(set(s))

vals = numpy.array([255 for _ in range(38)])
vals[reduce] = numpy.array(range(0, 32))
if count(vals) != 32:
    print('Something went wrong')
    sys.exit(1)
print(repr(vals))
