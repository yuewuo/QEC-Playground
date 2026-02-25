import sys, json


def print_decoding_hypergraph_info(filename: str):
    with open(filename, "r", encoding="utf8") as f:
        data = json.load(f)
        model_hypergraph = data["model_hypergraph"]
        edge_num = len(model_hypergraph["edge_indices"])
        assert edge_num == len(model_hypergraph["weighted_edges"])
        vertex_num = len(model_hypergraph["vertex_indices"])
        print(f"{vertex_num} vertices, {edge_num} edges")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: <json visualizer filename>")
        exit(0)

    filename = sys.argv[1]
    print_decoding_hypergraph_info(filename)
