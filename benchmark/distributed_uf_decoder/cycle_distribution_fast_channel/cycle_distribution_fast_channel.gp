set terminal postscript eps color "Arial, 28"
set xlabel "Execution Clock Cycle" font "Arial, 28"
set ylabel "Probability" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:
# cargo run --release -- tool distributed_union_find_decoder_standard_planar_benchmark [3,5,7,9,11,13] [1e-2] -p0 -b100 -m100000000 -e100000000 --only_count_logical_x --output_cycle_distribution --fast_channel_interval 2
# cargo run --release -- tool distributed_union_find_decoder_standard_planar_benchmark [15,17,19] [1e-2] -p0 -b100 -m10000000000 -e100 --only_count_logical_x --output_cycle_distribution --fast_channel_interval 2

set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"


set style data histogram
# set style histogram clustered gap 0.5

# boxwidth=0.8

set style fill solid 0.2 border 1
# set boxwidth 1.0 absolute
set style data boxes

set output "cycle_distribution_fast_channel.eps"

everysome(col) = (int(column(col))%50 ==0)?stringcolumn(1):""

# plot "duf_15_0.01.txt" using 2:xticlabels(everysome(1)) title "d = 15" lc rgbcolor "blue"
# plot "duf_17_0.01.txt" using 2:xticlabels(everysome(1)) title "d = 17" lc rgbcolor "blue"
plot "duf_11_0.01.txt" using 2:xticlabels(everysome(1)) title "d = 11" lc rgbcolor "blue"

system("ps2pdf -dEPSCrop cycle_distribution_fast_channel.eps cycle_distribution_fast_channel.pdf")

set size 1,0.75
set output "cycle_distribution_fast_channel_w.eps"
replot
system("ps2pdf -dEPSCrop cycle_distribution_fast_channel_w.eps cycle_distribution_fast_channel_w.pdf")

set size 1,0.6
set output "cycle_distribution_fast_channel_w_w.eps"
replot
system("ps2pdf -dEPSCrop cycle_distribution_fast_channel_w_w.eps cycle_distribution_fast_channel_w_w.pdf")
