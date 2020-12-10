from graphviz import Graph

g = Graph('MWPM graph')

number = 7

def b(i):
    return 'b%d' % i
def e(i):
    return 'e%d' % i

for i in range(number):
    g.node(name=b(i), color='blue')
    g.node(name=e(i), color='red')
    g.edge(b(i), e(i), color='black')

for i in range(number):
    for j in range(i+1, number):
        g.edge(b(i), b(j), color='green')
        g.edge(e(i), e(j), color='purple')

g.view()
