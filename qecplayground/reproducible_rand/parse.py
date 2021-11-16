

def parse_file_to_array_buffer(filename):
    with open(filename, "rb") as f:
        bytes_vec = [int(b) for b in f.read()]
        print("new Uint8Array(%s)" % (bytes_vec))

if __name__ == "__main__":
    parse_file_to_array_buffer("index.wasm")
