# Fusion UF decoder with Time Partition

We'll check how fusion UF changes the logical error rate when the partition boundary is in time domain.

My hypothesis is that as long as the partition boundary is perpendicular to the logical error boundary,
then the effect on the logical error rate is small.
By perpendicular, it could be either time slice or one kind of spatial slice.
For simplicity, here we only investigate the time partition.

We investigate two cases:
1. various block height, changing from 1 to $2d$
2. fixed block height of $d$ and varying the code distance and physical error rate

## Graph Partition Tool

Since the configuration of fusion partition could be hard, I develop this general tool to partition the decoding graph.

`fusion-blossom/scripts/`

## $T = 2 * d$

We study two noise models `phenomenological_T2d` and `circuit_level_T2d`.
For each noise model, we look at four decoders: `fusion_mwpm`, `fusion_uf`, `mwpm` and `uf`.


