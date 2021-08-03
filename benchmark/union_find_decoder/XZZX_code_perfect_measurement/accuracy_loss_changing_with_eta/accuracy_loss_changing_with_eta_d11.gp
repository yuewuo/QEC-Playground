set terminal postscript eps color "Arial, 28"
set xlabel "{/Symbol h}" font "Arial, 28"
set ylabel "Logical Error Rate (p_L)" font "Arial, 28"
set grid ytics
set size 1,1

# data generating commands:

# UF
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 0.5 --decoder UF --max_half_weight 10 > uf_eta0.5.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 1 --decoder UF --max_half_weight 10 > uf_eta1.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 3 --decoder UF --max_half_weight 10 > uf_eta3.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 10 --decoder UF --max_half_weight 10 > uf_eta10.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 30 --decoder UF --max_half_weight 10 > uf_eta30.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 100 --decoder UF --max_half_weight 10 > uf_eta100.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 300 --decoder UF --max_half_weight 10 > uf_eta300.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 1000 --decoder UF --max_half_weight 10 > uf_eta1000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 3000 --decoder UF --max_half_weight 10 > uf_eta3000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 10000 --decoder UF --max_half_weight 10 > uf_eta10000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta +inf --decoder UF --max_half_weight 10 > uf_etainf.log 2>&1 &

# MWPM
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 0.5 > mwpm_eta0.5.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 1 > mwpm_eta1.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 3 > mwpm_eta3.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 10 > mwpm_eta10.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 30 > mwpm_eta30.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 100 > mwpm_eta100.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 300 > mwpm_eta300.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 1000 > mwpm_eta1000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 3000 > mwpm_eta3000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta 10000 > mwpm_eta10000.log 2>&1 &
# nohup ./rust_qecp tool fault_tolerant_benchmark [11,17] [0,0] [0.1] -p3-m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --bias_eta +inf > mwpm_etainf.log 2>&1 &

set logscale x
set xrange [0.5:100000]
set xtics ('1' 1, '1e1' 10, '1e2' 100, '1e3' 1000, '1e4' 10000, '{/Symbol \245}' 100000)
set logscale y
set ytics ("0.001" 0.001, "0.003" 0.003, "0.03" 0.03, "0.03" 0.03, "0.1" 0.1)
set yrange [0.001:0.1]
set key outside horizontal top center font "Arial, 24"

set style fill transparent solid 0.2 noborder

set title "XZZX Surface Code d = 11, p = 0.1"

set output "accuracy_loss_changing_with_eta_d11.eps"

plot "mwpm_d11.txt" using 1:7 with linespoints lt rgb "red" linewidth 5 pointtype 6 pointsize 1.5 title "MWPM Decoder",\
    "uf_d11.txt" using 1:6 with linespoints lt rgb "blue" linewidth 5 pointtype 2 pointsize 1.5 title "UnionFind Decoder"

set output '|ps2pdf -dEPSCrop accuracy_loss_changing_with_eta_d11.eps accuracy_loss_changing_with_eta_d11.pdf'
replot

set size 1,0.75
set output "accuracy_loss_changing_with_eta_d11_w.eps"
replot
set output '|ps2pdf -dEPSCrop accuracy_loss_changing_with_eta_d11_w.eps accuracy_loss_changing_with_eta_d11_w.pdf'
replot

set size 1,0.6
set output "accuracy_loss_changing_with_eta_d11_w_w.eps"
replot
set output '|ps2pdf -dEPSCrop accuracy_loss_changing_with_eta_d11_w_w.eps accuracy_loss_changing_with_eta_d11_w_w.pdf'
replot
