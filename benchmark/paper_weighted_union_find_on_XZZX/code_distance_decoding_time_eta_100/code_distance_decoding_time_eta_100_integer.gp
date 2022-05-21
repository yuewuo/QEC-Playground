set terminal postscript eps color "Arial, 22"
set xlabel "Code distance" font "Arial, 22"
set ylabel "Decoding time (s)" font "Arial, 22"
set grid ytics
set size 1,1.1

set logscale x
set xrange [2.9:21]
# print(", ".join([f"'{i}' {i}" for i in range(3,13)]))
set xtics ('3' 3, '4' 4, '5' 5, '6' 6, '8' 8, '10' 10, '12' 12, '16' 16, '20' 20)
set logscale y
# print(", ".join([f"'1e{i}' 1e{i}" for i in range(-4, 2)]))
set ytics ('1e-7' 1e-7, '1e-6' 1e-6, '1e-5' 1e-5, '1e-4' 1e-4, '1e-3' 1e-3, '1e-2' 1e-2, '0.1' 1e-1, '1' 1e0, '10' 1e1)
set yrange [1e-7:1]
set key outside horizontal top center font "Arial, 22"

set style fill transparent solid 0.2 noborder
set key samplen 4
# set key maxrows 2
# set key height 5

set output "code_distance_decoding_time_eta_100_integer.eps"

# plot "processed_MWPM.txt" using 1:5 with linespoints lt rgb "red" linewidth 3 pointtype 7 pointsize 1 title "MWPM 1.1x",\
#     "" using 1:7 with linespoints lt rgb "blue" linewidth 3 pointtype 7 pointsize 1 title "MWPM 2x",\
#     "" using 1:10 with linespoints lt rgb "green" linewidth 3 pointtype 7 pointsize 1 title "MWPM Average",\

    slope0002 = "`tail -n1 processed_UF_integer_0.0002.txt | awk '{print $2}' | head -n1`"+0
intercept0002 = "`tail -n1 processed_UF_integer_0.0002.txt | awk '{print $3}' | head -n1`"+0
    slope0005 = "`tail -n1 processed_UF_integer_0.0005.txt | awk '{print $2}' | head -n1`"+0
intercept0005 = "`tail -n1 processed_UF_integer_0.0005.txt | awk '{print $3}' | head -n1`"+0
    slope001 = "`tail -n1 processed_UF_integer_0.001.txt | awk '{print $2}' | head -n1`"+0
intercept001 = "`tail -n1 processed_UF_integer_0.001.txt | awk '{print $3}' | head -n1`"+0
    slope002 = "`tail -n1 processed_UF_integer_0.002.txt | awk '{print $2}' | head -n1`"+0
intercept002 = "`tail -n1 processed_UF_integer_0.002.txt | awk '{print $3}' | head -n1`"+0
    slope004 = "`tail -n1 processed_UF_integer_0.004.txt | awk '{print $2}' | head -n1`"+0
intercept004 = "`tail -n1 processed_UF_integer_0.004.txt | awk '{print $3}' | head -n1`"+0
    slope008 = "`tail -n1 processed_UF_integer_0.008.txt | awk '{print $2}' | head -n1`"+0
intercept008 = "`tail -n1 processed_UF_integer_0.008.txt | awk '{print $3}' | head -n1`"+0

print intercept0002

# slope = 2.383367669061344, slope_avr = 1.2405810615278214, intercept = -11.881313999350098
fit3(x) = exp(2 * log(x) + -10)
# slope = 2.446655656145592, slope_avr = 1.3495294001474354, intercept = -10.895459524433363
fit2(x) = exp(2.446655656145592 * log(x) + (-10.895459524433363))
# slope = 2.374166520482226, slope_avr = 1.3077680697194967, intercept = -9.58705092892534
fit1(x) = exp(2.374166520482226 * log(x) + (-9.58705092892534))

# "" using 1:($5):($5-$6):($5+$6) with errorbars lt rgb "red" linewidth 3 pointtype 11 pointsize 1 notitle,\

plot "processed_UF_integer_0.0002.txt" using 1:($5) with linespoints lt rgb "red" linewidth 3 pointtype 11 pointsize 1 title "0.0002",\
    fit1(x) notitle with lines linestyle 2,\
    "processed_UF_integer_0.0005.txt" using 1:($5) with linespoints lt rgb "red" linewidth 3 pointtype 11 pointsize 1 title "0.0005",\
    fit1(x) notitle with lines linestyle 2,\
    "processed_UF_integer_0.001.txt" using 1:($5) with linespoints lt rgb "red" linewidth 3 pointtype 11 pointsize 1 title "0.001",\
    fit1(x) notitle with lines linestyle 2,\
    "processed_UF_integer_0.002.txt" using 1:($5) with linespoints lt rgb "red" linewidth 3 pointtype 11 pointsize 1 title "0.002",\
    fit1(x) notitle with lines linestyle 2,\
    "processed_UF_integer_0.004.txt" using 1:($5) with linespoints lt rgb "red" linewidth 3 pointtype 11 pointsize 1 title "0.004",\
    fit1(x) notitle with lines linestyle 2,\
    "processed_UF_integer_0.008.txt" using 1:($5) with linespoints lt rgb "red" linewidth 3 pointtype 11 pointsize 1 title "0.008",\
    fit1(x) notitle with lines linestyle 2

system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100_integer.eps code_distance_decoding_time_eta_100_integer.pdf")

# set size 1,0.75
# set output "code_distance_decoding_time_eta_100_integer_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100_integer_w.eps code_distance_decoding_time_eta_100_integer_w.pdf")

# set size 1,0.6
# set output "code_distance_decoding_time_eta_100_integer_w_w.eps"
# replot
# system("ps2pdf -dEPSCrop code_distance_decoding_time_eta_100_integer_w_w.eps code_distance_decoding_time_eta_100_integer_w_w.pdf")
