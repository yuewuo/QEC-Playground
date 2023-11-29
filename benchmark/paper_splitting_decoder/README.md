# Evaluation

In the original splitting decoder paper, it says that introducing the Y errors on the corner will reduce the
code distance. However, in my first attempt to reproduce this, I didn't find the effect. Although the logical 
error rate is indeed worse than the decoder without the Y edges.

This evaluation aims to answer this question in detail, including code capacity noise model, phenomenological noise
model and circuit-level noise model.
