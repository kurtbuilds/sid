import numpy

alphabet = "0abcdefghjkmnpqrstuvwxyz23456789"

orig = numpy.array([ord(a) for a in alphabet])

reduce = orig % 38

def count(alphabet):
    return len(set(alphabet))

vals = numpy.array([0 for _ in range(38)])
vals[reduce] = numpy.array(range(0, 32))
print(repr(vals))
