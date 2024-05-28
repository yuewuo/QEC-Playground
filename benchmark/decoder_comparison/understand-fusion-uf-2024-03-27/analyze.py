import json


fusion_failed = [19,24,51,96,217,275,302,333,337,384,415,418,440,480,524,638,743,777,841,852,1041,1073,1107,1138,1139,1184,1198,1268,1283,1327,1426,1429,1447,1483,1594,1600,1622,1639,1663,1716,1721,1736,1744,1879,1927,2029,2058,2069,2089,2108,2133,2253,2254,2318,2434,2582,2662,2804,2933,3002,3185,3218,3301,3330,3345,3353,3412,3460,3480,3507,3581,3605,3633,3677,3822,3832,3837,3853,3854,3873,3963,4009,4050,4241,4331,4363,4370,4451,4540,4555,4568,4741,4843,4848,4872,4887,5066,5134,5160,5173,5206,5218,5231,5261,5371,5407,5431,5434,5471,5535,5657,5676,5704,5706,5753,5834,5840,5852,5873,5917,6038,6123,6159,6167,6214,6243,6263,6329,6351,6370,6434,6467,6478,6486,6504,6622,6713,6755,6886,6898,6960,6964,7120,7176,7230,7243,7415,7448,7512,7530,7549,7631,7738,7779,7818,7827,7883,7885,7960,8024,8089,8106,8115,8119,8164,8212,8284,8333,8344,8441,8469,8487,8498,8653,8713]
uf_failed = [24,51,96,207,217,275,302,317,333,337,384,415,418,440,480,524,533,638,743,777,852,981,1041,1073,1080,1107,1138,1139,1184,1191,1198,1283,1327,1426,1427,1429,1447,1483,1536,1594,1600,1614,1622,1639,1663,1668,1716,1721,1736,1744,1879,1927,2029,2058,2069,2108,2133,2150,2253,2254,2317,2318,2434,2582,2662,2785,2804,2933,3002,3185,3187,3218,3301,3330,3345,3412,3460,3480,3507,3581,3605,3633,3677,3777,3822,3832,3837,3853,3854,3873,3963,4050,4150,4241,4331,4451,4540,4555,4568,4741,4843,4848,4872,4887,4893,4925,5066,5079,5160,5173,5206,5218,5231,5261,5371,5407,5431,5434,5471,5483,5531,5535,5657,5676,5704,5706,5753,5834,5840,5852,5873,5898,5917,6038,6123,6159,6167,6214,6243,6273,6289,6329,6351,6370,6467,6478,6486,6504,6622,6713,6755,6886,6898,6960,6964,7002,7120,7176,7230,7280,7415,7448,7473,7512,7530,7549,7630,7631,7738,7742,7779,7818,7827,7835,7883,7960,8089,8106,8115,8119,8164,8212,8284,8333,8344,8469,8487,8498,8653,8713,8832,8838,9118,9148,9212,9217,9332,9403,9534,9541,9574,9575,9600,9665,9727,9826,9975]

print(f"fusion_failed: {len(fusion_failed)}")
print(f"uf_failed: {len(uf_failed)}")

print()

fusion_only = [e for e in fusion_failed if e not in uf_failed]
uf_only = [e for e in uf_failed if e not in fusion_failed]
shared = [e for e in fusion_failed if e in uf_failed]

print(f"fusion_only: {len(fusion_only)}")
print(f"uf_only: {len(uf_only)}")
print(f"shared: {len(shared)}")

with open("fusion.stats", "r", encoding='utf8') as f:
    lines = f.readlines()
    print(len(lines))
    assert(len(lines) == 10002)

def print_failed_pattern(failures: list[int], filename: str):
    # read all lines
    entries = dict()
    with open(filename, "r", encoding='utf8') as f:
        lines = f.readlines()
        for line in lines:
            line = line.strip("\r\n ")
            if line == "":
                continue
            prefix = line.split(":")[0]
            index = int(prefix)
            entries[index] = json.loads(line[len(prefix)+1:])
    for index in failures:
        print(f"[{index}]: {entries[index]['error_pattern']}")

print("fusion only failures:")
print_failed_pattern(fusion_only, "fusion.failed")

print("UF only failures:")
print_failed_pattern(uf_only, "uf.failed")

print("shared failures:")
print_failed_pattern(shared, "uf.failed")

def filter_syndrome(indices: list[int], syndrome_filename: str, output_filename: str):
    with open(syndrome_filename, "r", encoding='utf8') as f:
        lines = f.readlines()
    header = lines[0:3]
    assert(len(lines) == 10003)
    with open(output_filename, "w", encoding='utf8') as f:
        for head in header:
            f.write(head)
        for index in indices:
            snap_index = index - 2  # start at line 2
            line = lines[snap_index + 3 - 1].strip("\r\n ")  # start at line 3
            f.write(line + '\n')

filter_syndrome(fusion_only, "syndrome.txt", "fusion_only.syndrome")
filter_syndrome(uf_only, "syndrome.txt", "uf_only.syndrome")
filter_syndrome(shared, "syndrome.txt", "shared.syndrome")
