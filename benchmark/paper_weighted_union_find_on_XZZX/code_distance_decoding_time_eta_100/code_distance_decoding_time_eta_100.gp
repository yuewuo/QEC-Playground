set terminal postscript eps color "Arial, 22"
set xlabel "Code distance d, code patch d*(3d) with 100 measurements" font "Arial, 22"
set ylabel "Decoding time per measurement round (s)" font "Arial, 22"
set grid ytics
set size 1,1.1

set logscale x
set xrange [2.9:25]
# print(", ".join([f"'{i}' {i}" for i in range(3,13)]))
set xtics ('3' 3, '4' 4, '5' 5, '6' 6, '8' 8, '10' 10, '12' 12, '16' 16, '20' 20, '24' 24)
set logscale y
# print(", ".join([f"'1e{i}' 1e{i}" for i in range(-4, 2)]))
set ytics ('1e-4' 1e-4, '1e-3' 1e-3, '1e-2' 1e-2, '0.1' 1e-1, '1' 1e0, '10' 1e1)
set yrange [1e-7:2e-3]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder
set key samplen 4
set key maxrows 2
# set key height 5

set output "code_distance_decoding_time_eta_100.eps"

# plot "processed_MWPM.txt" using 1:5 with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "MWPM 1.1x",\
#     "" using 1:7 with linespoints lt rgb "blue" linewidth 3 pointtype 7 pointsize 1 title "MWPM 2x",\
#     "" using 1:10 with linespoints lt rgb "green" linewidth 3 pointtype 7 pointsize 1 title "MWPM Average",\

# slope = 2.383367669061344, slope_avr = 1.2405810615278214, intercept = -11.881313999350098
fit3(x) = exp(2 * log(x) + -10)
# slope = 2.446655656145592, slope_avr = 1.3495294001474354, intercept = -10.895459524433363
fit2(x) = exp(2.446655656145592 * log(x) + (-10.895459524433363))
# slope = 2.374166520482226, slope_avr = 1.3077680697194967, intercept = -9.58705092892534
fit1(x) = exp(2.374166520482226 * log(x) + (-9.58705092892534))

plot "processed_UF_0.0002.txt" using 1:($5/100) with linespoints lt rgb "red" linewidth 3 pointtype 11 pointsize 1 title "UF 0.003",\
    "" using 1:($5/100):(($5-$6)/100):(($5+$6)/100) with errorbars lt rgb "red" linewidth 3 pointtype 11 pointsize 1 notitle,\
    fit1(x)/100 notitle with lines linestyle 2
    # "processed_UF_0.001.txt" using 1:($5/100) with linespoints lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 title "UF 0.001",\
    # "" using 1:($5/100):(($5-$6)/100):(($5+$6)/100) with errorbars lt rgb "blue" linewidth 3 pointtype 11 pointsize 1 notitle,\
    # fit2(x)/100 notitle with lines linestyle 2,\
    # "processed_UF_0.0003.txt" using 1:($5/100) with linespoints lt rgb "green" linewidth 3 pointtype 11 pointsize 1 title "UF 0.0003",\
    # "" using 1:($5/100):(($5-$6)/100):(($5+$6)/100) with errorbars lt rgb "green" linewidth 3 pointtype 11 pointsize 1 notitle,\
    # fit3(x)/100 notitle with lines linestyle 2

system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100.eps code_distance_decoding_time_eta_100.pdf")

# set size 1,0.75
# set output "code_distance_decoding_time_eta_100_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100_w.eps code_distance_decoding_time_eta_100_w.pdf")

# set size 1,0.6
# set output "code_distance_decoding_time_eta_100_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100_w_w.eps code_distance_decoding_time_eta_100_w_w.pdf")
