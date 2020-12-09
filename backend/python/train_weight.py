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

from PIL import Image
from tinynn.core.layer import Dense
from tinynn.core.layer import ReLU
from tinynn.core.layer import Sigmoid
from tinynn.core.loss import MSE
from tinynn.core.model import Model
from tinynn.core.net import Net
from tinynn.core.optimizer import Adam
from tinynn.utils.data_iterator import BatchIterator
from tinynn.utils.metric import mean_square_error
from tinynn.utils.seeder import random_seed

from tinynn.core.loss import Loss


def load_data():
    def distance_delta(i, j):
        return (abs(i + j) + abs(i - j)) / 2.

    d = 5

    edges = np.zeros((1, (d + 1) * (d + 1)))
    weights = np.zeros((1, (d + 1) * (d + 1)))
    for i in range(d + 1):
        for j in range(d + 1):
            edges[0, i * (d + 1) + j] = (i * (d + 1) + j) / float(d + 1) / float(d + 1)
            weights[0, i * (d + 1) + j] = distance_delta(i, j)

    return edges, weights


def weights_to_loss(weights):
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

    return compute_error_rate(nweights, min_error_cases=10)

class LMWPM(Loss):
    def __init__(self, lr):
        self.last_loss = 0.0
        self.curr_loss = 0.0
        self.lr = lr
        self.init_inputs, self.init_weights = load_data()
        
    def loss(self, predicted, actual):
        # print("predicted shape: {}".format(predicted.shape))
        er = weights_to_loss(predicted + self.init_weights - 0.5)
        self.curr_loss = er
        print("error rate: {}".format(er))
        return er
    
    def grad(self, predicted, actual):
        m = 10000000. * self.lr * (self.curr_loss - self.last_loss)
        print("grad: {}".format(m))
        self.last_loss = self.curr_loss
        return m


def main(batch_size, epochs, lr, logs_dir, seed):
    """
    Main function that performs training and test on a validation set
    :param weight_file: weight input file with training data
    :param batch_size: batch size to use at training time
    :param epochs: number of epochs to train for
    :param lr: learning rate
    :param val: percentage of the training data to use as validation
    :param logs_dir: directory where to save logs and trained parameters/weights
    """

    if seed >= 0:
        random_seed(seed)

    print("Importing weights...")
    input, target = load_data()

    print("input shape: {}, target shape: {}".format(input.shape, target.shape))

    N = input.shape[0]
    assert N == target.shape[0], \
        "The input and target arrays had different amounts of data ({} vs {})".format(N, target.shape[0]) # sanity check!
    print("Loaded {} training examples.".format(N))

    net = Net([
        Dense(32),
        ReLU(),
        Dense(128),
        ReLU(),
        Dense(64),
        ReLU(),
        Dense(36),
        Sigmoid()
    ])

    model = Model(net=net, loss=LMWPM(lr=lr), optimizer=Adam())

    
    iterator = BatchIterator(batch_size=batch_size)
    for epoch in range(epochs):
        for batch in iterator(input, target):
            # print("batch input shape: {}, batch target shape: {}".format(batch.inputs, batch.targets))
            preds = model.forward(batch.inputs)
            loss, grads = model.backward(preds, batch.targets)
            model.apply_grads(grads)

        # evaluate
        preds = net.forward(input)
        mse = mean_square_error(preds, target)
        print("Epoch %d %s" % (epoch, mse))



if __name__ == "__main__":
    # script arguments
    parser = argparse.ArgumentParser()
    parser.add_argument("--epochs", help="number of epochs for training",
                        type=int, default=10000)
    parser.add_argument("--batch_size", help="batch size used for training",
                        type=int, default=1)
    parser.add_argument("--lr", help="learning rate for training",
                        type=float, default=1e-3)
    # parser.add_argument("--val", help="percent of training data to use for validation",
    #                     type=float, default=0.8)
    # parser.add_argument("--input", help="input weights",
    #                     type=str, required=True)
    parser.add_argument("--logs_dir", help="logs directory",
                        type=str, default="")
    parser.add_argument("--seed", default=-1, type=int)
    args = parser.parse_args()

    if len(args.logs_dir) == 0: # parameter was not specified
        args.logs_dir = 'logs/log_{}'.format(datetime.datetime.now().strftime("%m-%d-%Y-%H-%M"))

    if not os.path.isdir(args.logs_dir):
        os.makedirs(args.logs_dir)

    # # run the main function
    main(args.batch_size, args.epochs, args.lr, args.logs_dir, args.seed)
    # sys.exit(0)

    # default_weights = generate_weights_from_function(5, default_weights)
    # error_rate = compute_error_rate(default_weights, min_error_cases=10)
    # print("error_rate:", error_rate)