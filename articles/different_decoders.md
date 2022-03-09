# Different decoders

Clarify at first: the complexity of decoding problem depends on:

1. Total measurement rounds: $m$
2. code distance: $d$
3. How many errors in each round: $n$
   1. so the total number of errors $N = m n$
   2. fixed physical error rate, $n=O(d^2)$, $N=O(md^2)$
4. error model:
   1. whether it's perfect measurement or not
   2. whether it's biased noise or not
   3. other special attributes that will change the graph structure

Here I give some decoder's **worst case complexity** (their average complexity is usually the same ~$O(N)$)

- A: union-find decoder:
  - $O(md^2)$ when imperfect measurement
  - $O(md^2)$ when perfect measurement
  - $O(md^2)$ when perfect measurement + biased noise
- B: distributed union-find decoder with #(processing units) = $O(d^3)$:
  - $O(md + d^3)$ when imperfect measurement
  - $O(md + d^2)$ when perfect measurement
  - $O(md + d)$ when perfect measurement + biased noise
- C: MWPM decoder (using Blossom V with complexity $O(|V||E|\log{|V|})$):
  - $O(md^2 md^2 d)=O(m^2 d^5)$ when imperfect measurement
  - $O(md^2 d^2)=O(md^4)$ when perfect measurement
  - $O(md^2 d)=O(md^3)$ when perfect mesurement + biased noise
- D: Lin's assumed decoder (which is always $O(N^2)$ complexity, taking no information of error model):
  - $O(N^2)=O(m^2d^4)$ when imperfect measurement
  - $O(N^2)=O(m^2d^4)$ when perfect measurement
  - $O(N^2)=O(m^2d^4)$ when perfect measurement + biased noise

Here are a few options to run each decoder

### 1. Batch decoding

##### imperfect measurement:



