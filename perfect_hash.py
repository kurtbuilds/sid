import numpy

alphabet = "023456789abcdefghjkmnpqrstuvwxyz"

orig = numpy.array([ord(a) for a in alphabet])

reduce = orig % 38

def count(alphabet):
    return len(set(alphabet))

vals = numpy.array([0 for _ in range(38)])
vals[reduce] = numpy.array(range(0, 32))
print(repr(vals))
