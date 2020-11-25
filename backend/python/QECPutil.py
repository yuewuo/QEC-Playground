#!/usr/bin/python3
import numpy as np
import os, json, math


"""
load measurement or ground truth data from file
The format of file should be <JSON head> 0x00 <layer> x N,
    where layer should be ceil(L^2 / 8) bytes.
This API is not intended to be used on large files, because it simply load data into memory. It takes more than 8 times larger memory when loaded.

The return value is a pair of (head, data), where head is a `dict` object corresponding to the JSON head in the file, and data is a numpy array of [N][L][L] (boolean)

Note that we don't need to align to 8 bytes because this file format is condensed and is not expected to use directly.
"""
def load(filepath):
    with open(filepath, "rb") as f:
        file_bytes = f.read()
    split_idx = file_bytes.find(b"\0")
    file_array = np.frombuffer(file_bytes, dtype=np.uint8)
    head_array = file_array[:split_idx]
    data_array = file_array[split_idx+1:]
    head = json.loads(str(head_array, encoding="ascii"))
    N = head["N"]
    L = head["L"]
    assert N > 0 and L > 0
    cycle = math.ceil(L*L/8)
    assert len(data_array) > 0 and len(data_array) == cycle * N
    data = np.zeros((N, L, L), dtype=np.bool)
    for i in range(N):
        base_idx = i * cycle
        l = 0
        for j in range(L):
            for k in range(L):
                byte_idx = base_idx + l // 8
                bit_idx = l % 8
                data[i][j][k] = data_array[byte_idx] & (1 << bit_idx)
                l += 1
    return head, data

""""
save measurement or ground truth data to file using the same format as `load`
"""
def save(filepath, head, data):
    head_str = json.dumps(head)
    assert len(data.shape) == 3, "should be data[N][L][L]"
    N, L, L2 = data.shape
    assert L == L2, "should be data[N][L][L]"
    
    # deep copy the head and filter out python specific attributes
    head = json.loads(head_str)
    assert not "N" in head, "avoid providing `N` in head, which will be added automatically"
    head["N"] = N
    assert not "L" in head, "avoid providing `L` in head, which will be added automatically"
    head["L"] = L
    head_str = json.dumps(head)

    head_bytes = bytes(head_str, "ascii")
    cycle = math.ceil(L*L/8)
    data_length = N * cycle
    data_array = np.zeros((data_length,), dtype=np.uint8)
    for i in range(N):
        base_idx = i * cycle
        l = 0
        for j in range(L):
            for k in range(L):
                if data[i,j,k] == True:
                    byte_idx = base_idx + (l // 8)
                    bit_idx = l % 8
                    data_array[byte_idx] |= 1 << bit_idx
                l += 1
    data_bytes = bytes(data_array)
    with open(filepath, 'wb') as f:
        f.write(head_bytes)
        f.write(b'\0')
        f.write(data_bytes)
