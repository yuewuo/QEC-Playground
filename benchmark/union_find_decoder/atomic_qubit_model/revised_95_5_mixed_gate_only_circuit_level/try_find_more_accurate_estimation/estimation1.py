import math

"""
Given odd d

pL = A1 * [
    d / 2 * Sum[C[i, i+d-2*i] * pe ^ (d-2i) * pp ^ i, {i, 0, (d-1)/2}]
  + 2 * d^2 * Sum[C[i, i+d+1-2*i] * pe ^ (d+1-2*i) * pp ^ i, {i, 0, (d+1)/2}]
]
"""

def nCr(n,r):
    f = math.factorial
    return f(n) // f(r) // f(n-r)
def nPr(n,r):
    f = math.factorial
    return f(n) // f(n-r)

def estimate(d, p_erasure, p_pauli, tune_A = 1.0, tune_E = 1.0, tune_P = 1.0, possibility_each_turning = 4):
    assert d % 2 == 1, "d must be odd number"
    pe = p_erasure * tune_E
    pp = p_pauli * tune_P
    pL = 0
    for j in range((d - 1) // 2):
        combination = d * d  # number of different left starting point
        combination *= nPr(d, j) # number of different turning point choices
        combination *= (possibility_each_turning ** j) # each turning point has 4 options in 3D lattice (up down left right)
        for i in range((d - 1) // 2 + j):
            pL += combination * nCr(i+d+j-2*i, i) * (pe ** (d+j-2*i) * (pp ** i))
    pL *= tune_A  # tune ratio
    return pL





# print(estimate(3, 0.005, 0.0005))


for d in [3,5,7,9]:
    for p in [0.0315 * (0.95 ** i) for i in range(100)]:
        if p >= 0.02:
            continue
        # estimated_pL = estimate(d, p * 0.95, p * 0.05, tune_E = 4, tune_P = 4, possibility_each_turning = 8)
        estimated_pL=  estimate(d, p * 0.95, p * 0.05, tune_A=11.4404, tune_E = 6.72476, tune_P = 3.30978, possibility_each_turning = 8)
        if estimated_pL < 1e-8:
            break
        print(d, p, estimated_pL)
    print("")
