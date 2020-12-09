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


def load_data():
    def distance_delta(i, j):
        return (abs(i + j) + abs(i - j)) / 2.
    d = 5

    edges = np.zeros((1, (d + 1) * (d + 1)))
    weights = np.zeros((1, (d + 1) * (d + 1)))
    for i in range(d + 1):
        for j in range(d + 1):
            edges[0, i * (d + 1) + j] = (i * (d + 1) + j) / float(d + 1) / float(d + 1)
            weights[0, i * (d + 1) + j] = distance_delta(i, j) + 1 # add 1 bias

    return edges, weights


def weights_to_loss(weights, debug=False):
    if debug:
        print("weights: ")
        print(weights)
    d = 5
    nweights = np.zeros((d + 1, d + 1, d + 1, d + 1))
    for i1 in range(d + 1):
        for j1 in range(d + 1):
            for i2 in range(d + 1):
                for j2 in range(d + 1):
                    di = i2 - i1
                    dj = j2 - j1
                    nweights[i1, j1, i2, j2] = weights[0, di * (d + 1) + dj]

    return compute_error_rate(nweights, min_error_cases=100, parallel=0)


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
        last_loss = weights_to_loss(target, True)
        filer.write(str(last_loss) + "\n")
        np.savetxt(filew, target)
        loss_list.append(last_loss)
        print("Epoch {}: loss = {}".format(epoch, last_loss))
        delta_loss = np.zeros((1, (d + 1) * (d + 1)))
        for i in range(d + 1):
            for j in range(d + 1):
                delta_target = np.copy(target)
                delta_target[0, i * (d + 1) + j] += gr
                delta_loss[0, i * (d + 1) + j] = (weights_to_loss(delta_target) - last_loss) / gr
        print("delta: ")
        print(delta_loss)
        target -= delta_loss * lr

    
    for x in loss_list:
        file.write(str(x) + "\n")
    np.savetxt(logs_dir + '/final_weights.txt', target)



if __name__ == "__main__":
    # script arguments
    parser = argparse.ArgumentParser()
    parser.add_argument("--epochs", help="number of epochs for training",
                        type=int, default=100)
    parser.add_argument("--lr", help="learning rate for training",
                        type=float, default=1e2)
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