set terminal postscript eps color "Arial, 28"
set xlabel "Code Distance" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

set xrange [3:13]
set xtics ("3" 3, "5" 5, "7" 7, "9" 9, "11" 11, "13" 13)
set logscale y
set ytics ("10^{-8}" 0.00000001, "10^{-7}" 0.0000001, "10^{-6}" 0.000001, "10^{-5}" 0.00001, "10^{-4}" 0.0001, "10^{-3}" 0.001, "10^{-2}" 0.01, "10^{-1}" 0.1)
set yrange [0.00000001:1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set output "compare_fix_physical_error_rate.eps"

plot "MWPM_decoder_1e-2.txt" using 2:6 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "MWPM p=1e-2",\
    "MWPM_decoder_2e-2.txt" using 2:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "MWPM p=2e-2",\
    "offer_decoder_1e-2.txt" using 2:5 with linespoints lt rgb "green" linewidth 5 pointtype 2 pointsize 1.5 title "offer p=1e-2",\
    "offer_decoder_2e-2.txt" using 2:5 with linespoints lt rgb "yellow" linewidth 5 pointtype 2 pointsize 1.5 title "offer p=2e-2"

system("ps2pdf -dEPSCrop compare_fix_physical_error_rate.eps compare_fix_physical_error_rate.pdf")

set size 1,0.75
set output "compare_fix_physical_error_rate_w.eps"
replot
system("ps2pdf -dEPSCrop compare_fix_physical_error_rate_w.eps compare_fix_physical_error_rate_w.pdf")

set size 1,0.6
set output "compare_fix_physical_error_rate_w_w.eps"
replot
system("ps2pdf -dEPSCrop compare_fix_physical_error_rate_w_w.eps compare_fix_physical_error_rate_w_w.pdf")
