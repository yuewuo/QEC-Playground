#!/usr/bin/env python
# Script to train and test a neural network with TF's Keras API for face detection

import os
import sys
import argparse
import datetime
import numpy as np

from MWPM_weighted import compute_error_rate
from MWPM_weighted import generate_weights_from_function
from MWPM_weighted import default_weights

def default_weights(i1, j1, i2, j2):
    def distance_delta(i, j):
        return (abs(i + j) + abs(i - j)) / 2.
    def distance(i1, j1, i2, j2):
        return distance_delta(i2 - i1, j2 - j1)
    return -distance(i1, j1, i2, j2)

def load_data():
    d = 5

    edges = np.zeros((d + 1, d + 1, d + 1, d + 1))
    weights = np.zeros((d + 1, d + 1, d + 1, d + 1))
    for i1 in range(d + 1):
        for j1 in range(d + 1):
            for i2 in range(d + 1):
                for j2 in range(d + 1):
                    edges[i1, j1, i2, j2] = (i1 * (d + 1)**3 + j1 * (d + 1)**2 + i2 * (d + 1) + j2) / (float(d + 1) ** 4)
                    weights[i1, j1, i2, j2] = default_weights(i1, j1, i2, j2) + 0 # add 1 bias

    return edges, weights


# def weights_to_loss(weights, debug=False):
#     if debug:
#         print("weights: ")
#         print(weights)
#     d = 5
#     nweights = np.zeros((d + 1, d + 1, d + 1, d + 1))
#     for i1 in range(d + 1):
#         for j1 in range(d + 1):
#             for i2 in range(d + 1):
#                 for j2 in range(d + 1):
#                     di = i2 - i1
#                     dj = j2 - j1
#                     nweights[i1, j1, i2, j2] = weights[0, di * (d + 1) + dj]

#     return compute_error_rate(nweights, min_error_cases=100, parallel=0)

def negative_weights_check(weights):
    d = 5
    for i1 in range(d + 1):
        for j1 in range(d + 1):
            for i2 in range(d + 1):
                for j2 in range(d + 1):
                    if weights[i1, j1, i2, j2] < 0:
                        weights[i1, j1, i2, j2] = 0

def main(epochs, lr, gr, logs_dir):
    """
    Main function that performs training and test on a validation set
    :param weight_file: weight input file with training data
    :param batch_size: batch size to use at training time
    :param epochs: number of epochs to train for
    :param lr: learning rate
    :param val: percentage of the training data to use as validation
    :param logs_dir: directory where to save logs and trained parameters/weights
    """

    d = 5

    print("Importing weights...")
    input, target = load_data()

    print("input shape: {}, target shape: {}".format(input.shape, target.shape))

    N = input.shape[0]
    assert N == target.shape[0], \
        "The input and target arrays had different amounts of data ({} vs {})".format(N, target.shape[0]) # sanity check!
    print("Loaded {} training examples.".format(N))

    loss_list = []

    file = open(logs_dir + "/loss.txt", "w+")
    filer = open(logs_dir + "/loss_running.txt", "w+")
    filew = open(logs_dir + "/running_weights.txt", "a")

    for epoch in range(epochs):
        last_loss = compute_error_rate(target, min_error_cases=100, parallel=0)
        filer.write(str(last_loss) + "\n")
        # np.savetxt(filew, target)
        loss_list.append(last_loss)
        print("Epoch {}: loss = {}".format(epoch, last_loss))
        delta_loss = np.zeros((d + 1, d + 1, d + 1, d + 1))
        delta_flag = np.zeros((d + 1, d + 1, d + 1, d + 1))
        cnt = 0
        for i1 in range(d + 1):
            for j1 in range(d + 1):
                for i2 in range(d + 1):
                    for j2 in range(d + 1):
                        if delta_flag[i1, j1, i2, j2] == 1:

                            continue
                        delta_target = np.copy(target)
                        delta_target[i1, j1, i2, j2] += gr
                        delta_target[j1, d-i1, j2, d-i2] += gr
                        delta_target[d-i1, d-j1, d-i2, d-j2] += gr
                        delta_target[d-j1, i1, d-j2, i2] += gr

                        delta_target[i2, j2, i1, j1] += gr
                        delta_target[j2, d-i2, j1, d-i1] += gr
                        delta_target[d-i2, d-j2, d-i1, d-j1] += gr
                        delta_target[d-j2, i2, d-j1, i1] += gr

                        delta_flag[i1, j1, i2, j2] = 1
                        delta_flag[j1, d-i1, j2, d-i2] = 1
                        delta_flag[d-i1, d-j1, d-i2, d-j2] = 1
                        delta_flag[d-j1, i1, d-j2, i2] = 1

                        delta_flag[i2, j2, i1, j1] = 1
                        delta_flag[j2, d-i2, j1, d-i1] = 1
                        delta_flag[d-i2, d-j2, d-i1, d-j1] = 1
                        delta_flag[d-j2, i2, d-j1, i1] = 1

                        tloss = (compute_error_rate(delta_target, min_error_cases=10, parallel=0) - last_loss) / gr
                        delta_loss[i1, j1, i2, j2] = tloss
                        delta_loss[j1, d-i1, j2, d-i2] = tloss
                        delta_loss[d-i1, d-j1, d-i2, d-j2] = tloss
                        delta_loss[d-j1, i1, d-j2, i2] = tloss

                        delta_loss[i2, j2, i1, j1] = tloss
                        delta_loss[j2, d-i2, j1, d-i1] = tloss
                        delta_loss[d-i2, d-j2, d-i1, d-j1] = tloss
                        delta_loss[d-j2, i2, d-j1, i1] = tloss
        print("delta: ")
        print(delta_loss)
        target -= delta_loss * lr
        negative_weights_check(target)

    
    for x in loss_list:
        file.write(str(x) + "\n")
    # np.savetxt(logs_dir + '/final_weights.txt', target)



if __name__ == "__main__":
    # script arguments
    parser = argparse.ArgumentParser()
    parser.add_argument("--epochs", help="number of epochs for training",
                        type=int, default=100)
    parser.add_argument("--lr", help="learning rate for training",
                        type=float, default=1e1)
    parser.add_argument("--gr", help="gradient rate for training",
                        type=float, default=1e-1)
    # parser.add_argument("--val", help="percent of training data to use for validation",
    #                     type=float, default=0.8)
    # parser.add_argument("--input", help="input weights",
    #                     type=str, required=True)
    parser.add_argument("--logs_dir", help="logs directory",
                        type=str, default="")
    args = parser.parse_args()

    if len(args.logs_dir) == 0: # parameter was not specified
        args.logs_dir = 'logs/log_{}'.format(datetime.datetime.now().strftime("%m-%d-%Y-%H-%M"))

    if not os.path.isdir(args.logs_dir):
        os.makedirs(args.logs_dir)

    # # run the main function
    main(args.epochs, args.lr, args.gr, args.logs_dir)
    # sys.exit(0)

    # default_weights = generate_weights_from_function(5, default_weights)
    # error_rate = compute_error_rate(default_weights, min_error_cases=10)
    # print("error_rate:", error_rate)