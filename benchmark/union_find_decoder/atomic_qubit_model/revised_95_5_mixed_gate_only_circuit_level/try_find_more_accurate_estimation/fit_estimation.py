from wolframclient.evaluation import WolframLanguageSession
from wolframclient.language import wl, wlexpr, Global
import math

# modify this to your wolfram alpha installation path if needed
# wlpath = "C:\\Program Files\\Wolfram Research\\Mathematica\\12.0\\MathKernel.exe"
# sess = WolframLanguageSession(wlpath)

session = WolframLanguageSession()  # use the default path

try:

    # define function
    session.evaluate("""
    nPr[n_, r_] := n!/(n - r)!;
    nCr[n_, r_] := Binomial[n, r];
    combination[d_, j_, pet_] := d * d * nPr[d, j] * Power[pet, j];
    estimate1[d_, pe_, pp_, tA_, tE_, tP_, pet_] := tA * Sum[combination[d, j, pet] * Sum[nCr[i+d+j-2*i, i] * Power[pe * tE, (d+j-2*i)] * Power[pp * tP, i], {i, 0, Floor[(d-1)/2] + j - 1}], {j, 0, Floor[(d-1)/2] - 1}];
    """)

    di_vec = [3,5,7,9]

    data = []
    for di in di_vec:
        raw_data_filename = f"raw_data_d_{di}.txt"
        with open(raw_data_filename, "r", encoding="utf8") as f:
            for line in f.readlines():
                line = line.strip("\r\n ")
                if line == "":
                    continue
                p, p_pauli, di, T, total_cnt, error_cnt, error_rate, dj, confidence_interval = line.split(" ")
                if float(p) < 0.02:
                    # ignore those around the threshold
                    data.append((int(di), float(p), float(error_rate)))

    data_expr = f"""{{{",".join([f"{{{d},{p:.12f},{math.log(error_rate):.6f}}}" for d,p,error_rate in data])}}}"""
    print(data_expr)

    # run this manually...
    # NMinimize[{Total[Map[(#[[3]] - Log[estimate1[#[[1]], #[[2]]*0.95, #[[2]]*0.05, tA, tE, tP, 4]])^2 &, data_expr]], tA > 0, tE > 0, tP > 0}, {tA, tE, tP}]



    # result = session.evaluate(f"""
    # NMinimize[{{Total[Map[(#[[3]] - Log[estimate1[#[[1]], #[[2]]*0.95, #[[2]]*0.05, tA, tE, tP, 4]])^2 &, data_expr]], tA > 0, tE > 0, tP > 0}}, {{tA, tE, tP}}]
    # """)
    # print(result)

    # has to terminate it at end, otherwise python wouldn't quit...
    session.terminate()

except Exception as e:
    # has to terminate it at end, otherwise python wouldn't quit...
    session.terminate()
    raise e
