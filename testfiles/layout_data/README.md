There are four different mixnet construction methods: 
-rand_rand, 
-rand_binpacking(rand_bp)
-bw_rand
-hybrid

For first three methods, I only consider the static setting where no nodes going down and back. Since mixnet will be reconstructed every epoch, there is no big different between static setting and dynamic setting.
From different stratigies of adversary, I pick the specific strategy that the adversary can achieve highest compromised traffic and record the corresponding topologies in 1000 epochs.

Fot the hybrid(Bow-tie) method, as we introduce the guard idea, considering static setting and dynamic setting seperately is necessary. There are one layout file in static setting and 6 layout files in dynamic setting. Each file has 1000 epochs topology information.


