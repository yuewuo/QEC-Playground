import os
import sys
import json
import matplotlib.pyplot as plt
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

if True:
    from common import *

script_folder = os.path.dirname(os.path.abspath(__file__))

plot_large(script_folder)
plot_relative(script_folder)
