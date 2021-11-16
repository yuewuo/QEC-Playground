
QECPlaygroundPath="/home/yw729/project/QEC-Playground"
# QECPlaygroundPath="/home/wuyue/QEC-Playground"

# test file output
# $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [4] --djs [12] [12] -m4000 -e100000 [0.005,0.0055,0.006] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithBiasedCX > biased_4.txt

SRUN="nohup srun -t 1-00:00:00 --mem=24G --mail-type=ALL --nodes=1 --ntasks=1 --cpus-per-task=36"
# SRUN="srun -t 1-00:00:00 --mem=4G --mail-type=ALL --nodes=1 --ntasks=1 --cpus-per-task=4"

# test srun
# $SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [4] --djs [12] [12] -m4000 -e100000 [0.005,0.0055,0.006] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithBiasedCX > biased_4.txt &

$SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [4] --djs [12] [12] -m400000 -e100000 [0.005,0.0055,0.006,0.0065,0.007,0.0075,0.008,0.0085,0.009,0.0095,0.01,0.0105,0.011] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithBiasedCX > biased_4.txt &
$SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [5] --djs [15] [15] -m400000 -e100000 [0.005,0.0055,0.006,0.0065,0.007,0.0075,0.008,0.0085,0.009,0.0095,0.01,0.0105,0.011] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithBiasedCX > biased_5.txt &
$SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [6] --djs [18] [18] -m400000 -e100000 [0.005,0.0055,0.006,0.0065,0.007,0.0075,0.008,0.0085,0.009,0.0095,0.01,0.0105,0.011] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithBiasedCX > biased_6.txt &
$SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [7] --djs [21] [21] -m400000 -e100000 [0.005,0.0055,0.006,0.0065,0.007,0.0075,0.008,0.0085,0.009,0.0095,0.01,0.0105,0.011] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithBiasedCX > biased_7.txt &
$SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [8] --djs [24] [24] -m400000 -e100000 [0.005,0.0055,0.006,0.0065,0.007,0.0075,0.008,0.0085,0.009,0.0095,0.01,0.0105,0.011] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithBiasedCX > biased_8.txt &
$SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [4] --djs [12] [12] -m400000 -e100000 [0.005,0.0055,0.006,0.0065,0.007,0.0075,0.008,0.0085,0.009,0.0095,0.01,0.0105,0.011] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithStandardCX > standard_4.txt &
$SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [5] --djs [15] [15] -m400000 -e100000 [0.005,0.0055,0.006,0.0065,0.007,0.0075,0.008,0.0085,0.009,0.0095,0.01,0.0105,0.011] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithStandardCX > standard_5.txt &
$SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [6] --djs [18] [18] -m400000 -e100000 [0.005,0.0055,0.006,0.0065,0.007,0.0075,0.008,0.0085,0.009,0.0095,0.01,0.0105,0.011] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithStandardCX > standard_6.txt &
$SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [7] --djs [21] [21] -m400000 -e100000 [0.005,0.0055,0.006,0.0065,0.007,0.0075,0.008,0.0085,0.009,0.0095,0.01,0.0105,0.011] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithStandardCX > standard_7.txt &
$SRUN $QECPlaygroundPath/backend/rust/target/release/rust_qecp tool fault_tolerant_benchmark [8] --djs [24] [24] -m400000 -e100000 [0.005,0.0055,0.006,0.0065,0.007,0.0075,0.008,0.0085,0.009,0.0095,0.01,0.0105,0.011] -p0 --time_budget 1800 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithStandardCX > standard_8.txt &
