set terminal postscript eps color "Arial, 28"
set xlabel "Number of qubits (n=2d(d-1))" font "Arial, 28"
set ylabel "Time to decode 10^6 samples (s)" font "Arial, 28"
# set grid ytics
set size 1,1

# set logscale x
set xrange [0:5000]
# labels
# python -c "for i in range(0, 5001, 200): print('\'%s\' %d' % (str(i) if i%1000==0 else '', i), end=', ')"
set xtics ('0' 0, '' 200, '' 400, '' 600, '' 800, '1000' 1000, '' 1200, '' 1400, '' 1600, '' 1800, '2000' 2000, '' 2200, '' 2400, '' 2600, '' 2800, '3000' 3000, '' 3200, '' 3400, '' 3600, '' 3800, '4000' 4000, '' 4200, '' 4400, '' 4600, '' 4800, '5000' 5000)
# set logscale y
set yrange [0:600]
# labels
# python -c "for i in range(0, 601, 20): print('\'%s\' %d' % (str(i) if i%100==0 else '', i), end=', ')"
set ytics ('0' 0, '' 20, '' 40, '' 60, '' 80, '100' 100, '' 120, '' 140, '' 160, '' 180, '200' 200, '' 220, '' 240, '' 260, '' 280, '300' 300, '' 320, '' 340, '' 360, '' 380, '400' 400, '' 420, '' 440, '' 460, '' 480, '500' 500, '' 520, '' 540, '' 560, '' 580, '600' 600)
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

# to remove legend (because I don't know how to plot it just like Fig.1 in arXiv1709.06218v1)
set nokey

set title "Decoding Time of CSS Surface Code (UF Decoder)"

set output "decoding_time_arXiv1709.06218v1.eps"

plot "decode_million_p0.01.txt" using 3:4 with points lt rgb "#5e81b5" linewidth 3 pointtype 7 pointsize 1.5 title "pz = 0.01",\
    "decode_million_p0.02.txt" using 3:4 with points lt rgb "#e19c24" linewidth 3 pointtype 5 pointsize 1.5 title "pz = 0.02"
    # "decode_million_p0.03.txt" using 3:4 with points lt rgb "#8fb032" linewidth 3 pointtype 13 pointsize 1.5 title "pz = 0.03",\
    # "decode_million_p0.04.txt" using 3:4 with points lt rgb "#eb6235" linewidth 3 pointtype 9 pointsize 1.5 title "pz = 0.04",\
    # "decode_million_p0.05.txt" using 3:4 with points lt rgb "#8778b3" linewidth 3 pointtype 11 pointsize 1.5 title "pz = 0.05",\

set output '|ps2pdf -dEPSCrop decoding_time_arXiv1709.06218v1.eps decoding_time_arXiv1709.06218v1.pdf'
replot

# set size 1,0.75
# set output "decoding_time_arXiv1709.06218v1_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop decoding_time_arXiv1709.06218v1_w.eps decoding_time_arXiv1709.06218v1_w.pdf'
# replot

# set size 1,0.6
# set output "decoding_time_arXiv1709.06218v1_w_w.eps"
# replot
# set output '|ps2pdf -dEPSCrop decoding_time_arXiv1709.06218v1_w_w.eps decoding_time_arXiv1709.06218v1_w_w.pdf'
# replot
