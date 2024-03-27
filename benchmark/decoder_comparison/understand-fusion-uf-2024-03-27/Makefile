
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
