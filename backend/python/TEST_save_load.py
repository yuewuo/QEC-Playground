import QECPutil
import numpy as np
import random

N = 1600
p = 1e-1
L = 15

# generate random errors on data
data = np.zeros((N, L, L), dtype=np.bool)
for i in range(N):
    for j in range(L):
        for k in range(L):
            data[i,j,k] = random.random() < p

head = {
    "p": p,
}

QECPutil.save("TEST.bin", head, data)
# head_r, data_r = QECPutil.load("TEST.bin")

# # assertions
# assert (data == data_r).all()
# for key in head:
#     assert head[key] == head_r[key]
