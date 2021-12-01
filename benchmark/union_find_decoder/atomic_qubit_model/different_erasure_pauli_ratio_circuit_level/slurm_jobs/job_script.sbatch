#!/bin/bash
#SBATCH --job-name=QEC-Playground=11-30-2021=17:46:41
#SBATCH --time=24:00:00
#SBATCH --mem=16G
#SBATCH --mail-type=ALL
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=36
#SBATCH --array=0-24
#SBATCH --out="/gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/slurm_jobs/%a.jobout"
#SBATCH --error="/gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/slurm_jobs/%a.joberror"

if [ "$SLURM_ARRAY_TASK_ID" == "0" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "1" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.01 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "2" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.02 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "3" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.03 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "4" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.04 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "5" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.05 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "6" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.1 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "7" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.15 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "8" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.2 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "9" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.25 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "10" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.3 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "11" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.35 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "12" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.4 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "13" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.45 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "14" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.5 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "15" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.55 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "16" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.6 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "17" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.65 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "18" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.7 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "19" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.75 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "20" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.8 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "21" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.85 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "22" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.9 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "23" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 0.95 || exit 91; fi
if [ "$SLURM_ARRAY_TASK_ID" == "24" ]; then python3 /gpfs/loomis/project/lin_zhong/yw729/QEC-Playground/benchmark/union_find_decoder/atomic_qubit_model/different_erasure_pauli_ratio_circuit_level/find_threshold.py 1 || exit 91; fi

