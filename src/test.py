from fastecdsa.curve import P256

point = P256.G * 5
print("({}, {})".format(point.x, point.y))
