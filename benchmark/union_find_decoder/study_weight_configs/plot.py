import numpy as np
import matplotlib.pyplot as plt
import os

# Plots
fig = plt.figure(figsize=(20,12))
ax0 = fig.add_subplot(2, 2, 1)
ax1 = fig.add_subplot(2, 2, 2)
ax2= fig.add_subplot(2, 2, 3)
ax3 = fig.add_subplot(2, 2, 4)

filename = os.path.join(os.path.dirname(__file__), f"thresholds.txt")

data = dict()
with open(filename, "r", encoding="utf8") as f:
    for line in f.readlines()[1:]:
        spt = line.split(" ")
        UF_decoder = eval(spt[0])
        no_autotune = eval(spt[1])
        autotune_minus_no_error = eval(spt[2])
        use_combined_probability = eval(spt[3])
        threshold = float(spt[6])
        relative_confidence_interval = float(spt[7])
        print(f"{UF_decoder} {no_autotune} {autotune_minus_no_error} {use_combined_probability} {threshold} {relative_confidence_interval}")
        data[(UF_decoder, no_autotune, autotune_minus_no_error, use_combined_probability)] = (threshold, relative_confidence_interval)


def name(UF_decoder=None, no_autotune=None, autotune_minus_no_error=None, use_combined_probability=None):
    name_vec = []
    if UF_decoder is not None:
        name_vec.append("UF" if UF_decoder else "MWPM")
    if no_autotune is not None:
        name_vec.append("weighted" if no_autotune else "unweighted")
    if autotune_minus_no_error is not None:
        name_vec.append("(1-p)/p" if autotune_minus_no_error else "1/p")
    if use_combined_probability is not None:
        name_vec.append("sum{p}" if use_combined_probability else "max{p}")
    return "\n".join(name_vec)

# see how decoder changes the result

def draw(ax, title, key, key_values):
    all_keys = ["UF_decoder", "no_autotune", "autotune_minus_no_error", "use_combined_probability"]
    assert key in all_keys, "should be a valid key"
    assert len(key_values) == 2
    key_idx = all_keys.index(key)
    data_1 = []
    data_2 = []
    tick_labels = []
    labels = []
    improvements = []
    for value in key_values:
        call_parameter = dict()
        call_parameter[key] = value
        labels.append(name(**call_parameter))
    for idx in range(2 ** len(all_keys)):
        if idx & (1 << key_idx) != 0:
            continue
        call_parameter = dict()
        data_key = []
        for ikey_idx, ikey in enumerate(all_keys):
            ikey_value = idx & (1 << ikey_idx) != 0
            data_key.append(ikey_value)
            if ikey != key:
                call_parameter[ikey] = ikey_value
        tick_labels.append(name(**call_parameter))
        data_key[key_idx] = key_values[0]
        data_1_value = data[tuple(data_key)]
        data_1.append(data_1_value)
        data_key[key_idx] = key_values[1]
        data_2_value = data[tuple(data_key)]
        data_2.append(data_2_value)
        improvements.append((data_2_value[0] - data_1_value[0]) / data_1_value[0])
    width = 0.3
    ax.bar(np.arange(len(tick_labels)), [e[0] for e in data_1], yerr=[e[1]*e[0]*2 for e in data_1], width=width, label=labels[0])
    ax.bar(np.arange(len(tick_labels)) + width, [e[0] for e in data_2], yerr=[e[1]*e[0]*2 for e in data_2], width=width, label=labels[1])
    ax.legend()
    ax.set_title(title + f" (average improvements = {100*sum(improvements)/len(improvements):.2f}%)")
    ax.set_xticks(np.arange(len(tick_labels)))
    ax.set_xticklabels(tick_labels)
    ax.set_ylabel('Threshold')
    ax.set_ylim([0,0.01])

draw(ax0, "Decoder Type", "UF_decoder", [True, False])
draw(ax1, "Autotune (weighted decoding graph)", "no_autotune", [True, False])
draw(ax2, "Weight Calculation", "autotune_minus_no_error", [True, False])
draw(ax3, "Probability Calculation", "use_combined_probability", [False, True])

# with ax1 as ax:
#     data_1 = []
#     data_2 = []
#     tick_labels = []
#     labels = [name(no_autotune=True), name(no_autotune=False)]
#     for UF_decoder in [False, True]:
#         for autotune_minus_no_error in [False, True]:
#             for use_combined_probability in [False, True]:
#                 tick_labels.append(name(UF_decoder=UF_decoder, autotune_minus_no_error=autotune_minus_no_error, use_combined_probability=use_combined_probability))
#                 data_1.append(data[(UF_decoder, True, autotune_minus_no_error, use_combined_probability)])
#                 data_2.append(data[(UF_decoder, False, autotune_minus_no_error, use_combined_probability)])
#     width = 0.3
#     ax.bar(np.arange(len(tick_labels)), [e[0] for e in data_1], yerr=[e[1]*e[0]*2 for e in data_1], width=width, label=labels[0])
#     ax.bar(np.arange(len(tick_labels)) + width, [e[0] for e in data_2], yerr=[e[1]*e[0]*2 for e in data_2], width=width, label=labels[1])
#     ax.legend()
#     ax.set_title("Autotune (weighted graph) Influence")
#     ax.set_ylabel('Threshold')
#     ax.set_ylim([0,0.01])



# for UF_decoder in [False, True]:
#     for no_autotune in [False, True]:
#         for autotune_minus_no_error in [False, True]:
#             for use_combined_probability in [False, True]:

# plot all figure
fig.tight_layout()
plt.show()
