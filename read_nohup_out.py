import sys, os

def read_nohup_out(filepath):
    with open(filepath, "rb") as f:
        lines = f.read().split(b"\n")
    for line in lines:
        last = line.split(b"\r")[-1]
        print(str(last, encoding="utf-8"))

if __name__ == "__main__":
    filepath = "nohup.out"
    if len(sys.argv) > 1:
        filepath = sys.argv[1]
    read_nohup_out(filepath)
