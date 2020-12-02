import networkx as nx

G = nx.Graph()

# G.add_nodes_from([0, 1, 2, 3])
# boundary_qubit = -10.
# qubit_qubit = -1.
# boundary_boundary = 0.
# G.add_weighted_edges_from([(0, 1, boundary_qubit), (1, 2, qubit_qubit), (2, 3, boundary_boundary), (3, 0, boundary_boundary)])

G.add_nodes_from([0, 1, 2, 3, 4, 5])
G.add_weighted_edges_from([
    (0, 1, -3.),
    (1, 2, -2.),
    (2, 0, -3.),
    (0, 3, -1.),
    (1, 4, -2.),
    (2, 5, -1.),
    (3, 4, 0.),
    (3, 5, 0.),
    (4, 5, 0.),
])

matching = nx.algorithms.matching.max_weight_matching(G, maxcardinality=True)
print(matching)
