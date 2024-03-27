
all: fusion uf syndrome

COMMAND := cargo run --release tool benchmark '[5]' '[0]' '[0.05]' -p1 --code-type rotated-planar-code --deterministic-seed 0
COMPARE_PARA := -m10000 --log-error-pattern-when-logical-error
ACCURACY_PARA := -e40000

fusion:
	$(COMMAND) $(COMPARE_PARA) --decoder fusion --decoder-config '{"max_tree_size":0}' --log-runtime-statistics ./fusion.stats

uf:
	$(COMMAND) $(COMPARE_PARA) --decoder union-find --decoder-config '{"max_half_weight":5000,"use_real_weighted":true}' --log-runtime-statistics ./uf.stats

syndrome:
	$(COMMAND) $(COMPARE_PARA) --decoder fusion --debug-print fusion-blossom-syndrome-file --fusion-blossom-syndrome-export-filename ./syndrome.txt

accuracy:
	@echo "MWPM"
	$(COMMAND) $(ACCURACY_PARA) --decoder fusion
	@echo "Fusion UF: max_tree_size = 0"
	$(COMMAND) $(ACCURACY_PARA) --decoder fusion --decoder-config '{"max_tree_size":0}'
	@echo "native UF"
	$(COMMAND) $(ACCURACY_PARA) --decoder union-find --decoder-config '{"max_half_weight":5000,"use_real_weighted":true}'

FUSION_VIZ_COMMAND := cargo run --release -- benchmark 5 0.05 --code-type error-pattern-reader --verifier none --enable-visualizer
FUSION_MWPM_PARA := --primal-dual-type parallel --primal-dual-config '{"primal":{"debug_sequential":true}}'
FUSION_UF_PARA := --primal-dual-type parallel --primal-dual-config '{"primal":{"debug_sequential":true,"max_tree_size":0}}'

FUSION_ONLY_SYNDROME := --code-config '{"filename":"../QEC-Playground/benchmark/decoder_comparison/understand-fusion-uf-2024-03-27/fusion_only.syndrome"}' -r 15
UF_ONLY_SYNDROME := --code-config '{"filename":"../QEC-Playground/benchmark/decoder_comparison/understand-fusion-uf-2024-03-27/uf_only.syndrome"}' -r 47
SHARED_SYNDROME := --code-config '{"filename":"../QEC-Playground/benchmark/decoder_comparison/understand-fusion-uf-2024-03-27/shared.syndrome"}' -r 160

FUSION_FOLDER ?= .

# you should run this in the fusion-blossom folder instead of the QEC-Playground folder
# FUSION_FOLDER=../../../../fusion-blossom make visualize

visualize: visualize-mwpm visualize-uf

visualize-mwpm:
	cd $(FUSION_FOLDER) && $(FUSION_VIZ_COMMAND) $(FUSION_ONLY_SYNDROME) $(FUSION_MWPM_PARA) --visualizer-filename fusion-only-mwpm.json
	cd $(FUSION_FOLDER) && $(FUSION_VIZ_COMMAND) $(UF_ONLY_SYNDROME) $(FUSION_MWPM_PARA) --visualizer-filename uf-only-mwpm.json
	cd $(FUSION_FOLDER) && $(FUSION_VIZ_COMMAND) $(SHARED_SYNDROME) $(FUSION_MWPM_PARA) --visualizer-filename shared-mwpm.json

visualize-uf:
	cd $(FUSION_FOLDER) && $(FUSION_VIZ_COMMAND) $(FUSION_ONLY_SYNDROME) $(FUSION_UF_PARA) --visualizer-filename fusion-only-uf.json
	cd $(FUSION_FOLDER) && $(FUSION_VIZ_COMMAND) $(UF_ONLY_SYNDROME) $(FUSION_UF_PARA) --visualizer-filename uf-only-uf.json
	cd $(FUSION_FOLDER) && $(FUSION_VIZ_COMMAND) $(SHARED_SYNDROME) $(FUSION_UF_PARA) --visualizer-filename shared-uf.json
