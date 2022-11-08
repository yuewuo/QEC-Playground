#!/bin/sh

# https://askubuntu.com/questions/50170/how-to-convert-pdf-to-image
# sudo apt install poppler-utils

pdftoppm circuit_level_noise_model_biased_only.pdf circuit_level_noise_model_biased_only -png -f 1 -singlefile -r 600
pdftoppm circuit_level_noise_model_biased_only_UF.pdf circuit_level_noise_model_biased_only_UF -png -f 1 -singlefile -r 600
