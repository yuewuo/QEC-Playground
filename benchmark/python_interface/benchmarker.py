import os, sys
from ssl import SSLSocket
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_benchmark_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout, compile_code_if_necessary
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities"))
import json, webbrowser, tempfile
from urllib import request, parse
from urllib.error import URLError, HTTPError

compile_code_if_necessary()

class Position:
    def __init__(self, t, i=None, j=None):
        if isinstance(t, str):
            assert t[0] == "[" and t[-1] == "]"
            spt = t[1:-1].split("][")
            self.t = int(spt[0])
            self.i = int(spt[1])
            self.j = int(spt[2])
        elif isinstance(t, int):
            self.t = t
            self.i = i
            self.j = j
        else:
            raise "unknown position format"

class QubitErrorModel:
    __slots__ = ("source", "position")  # prevent calling unknown attributes
    def __init__(self, source, position):
        self.source = source
        self.position = position

    def _get_is_virtual(self):
        return self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]["is_virtual"]
    is_virtual = property(
        fget=_get_is_virtual,
        doc="is this position a virtual qubit (physically non-existing one)"
    )

    def _get_is_peer_virtual(self):
        return self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]["is_peer_virtual"]
    is_peer_virtual = property(
        fget=_get_is_peer_virtual,
        doc="is the peer qubit is a virtual one"
    )

    def _get_t(self):
        return self.position.t
    t = property(
        fget=_get_t,
        doc="position of qubit in time axis"
    )

    def _get_i(self):
        return self.position.i
    i = property(
        fget=_get_i,
        doc="vertical position of qubit (top is 0)"
    )

    def _get_j(self):
        return self.position.j
    j = property(
        fget=_get_j,
        doc="horizontal position of qubit (left is 0)"
    )

    def _get_type(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        return node['qubit_type']
    type = property(
        fget=_get_type,
        doc="qubit type, e.g. Data, StabX, StabZ, StabXZZXLogicalX, StabXZZXLogicalZ, StabY... see `types::QubitType` for all"
    )
    qubit_type = property(
        fget=_get_type,
        doc="qubit type, e.g. Data, StabX, StabZ, StabXZZXLogicalX, StabXZZXLogicalZ, StabY... see `types::QubitType` for all"
    )

    def _get_peer(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            return None
        gate_peer = Position(node["gate_peer"])
        return self.source.at(gate_peer)
    peer = property(
        fget=_get_peer,
        doc="peer qubit object during a two-qubit gate, may be None if no such qubit"
    )

    def _get_gate_type(self):
        return self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]["gate_type"]
    gate = property(
        fget=_get_gate_type,
        doc="gate type, e.g. InitializeZ(X), CXGateControl, CXGateTarget, CYGateControl, CYGateTarget, CZGate, MeasureZ(X), None... see `simulator::GateType` for all"
    )
    gate_type = property(
        fget=_get_gate_type,
        doc="gate type, e.g. InitializeZ(X), CXGateControl, CXGateTarget, CYGateControl, CYGateTarget, CZGate, MeasureZ(X), None... see `simulator::GateType` for all"
    )

    def __repr__(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        alert = "\033[33;5m>\033[0m"  # yellow color
        # alert = ">"  # disable color
        position = Position(node['position'])
        rst = f"{node['qubit_type']} Qubit at [{position.t}][{position.i}][{position.j}]"
        if node['is_virtual']:
            rst += " (virtual)\n"
        else:
            rst += "\n"
        rst += f"    Single-Qubit Pauli Error:\n"
        rst += f"  {alert if node['error_model']['pp']['px'] > 0 else ' '}     pX = {node['error_model']['pp']['px']}\n"
        rst += f"  {alert if node['error_model']['pp']['pz'] > 0 else ' '}     pZ = {node['error_model']['pp']['pz']}\n"
        rst += f"  {alert if node['error_model']['pp']['py'] > 0 else ' '}     pY = {node['error_model']['pp']['py']}\n"
        rst += f"  {alert if node['error_model']['pp']['px']+node['error_model']['pp']['pz']+node['error_model']['pp']['py'] > 0 else ' '}       => pX+pZ+pY = {node['error_model']['pp']['px']+node['error_model']['pp']['pz']+node['error_model']['pp']['py']}\n"
        rst += f"    Single-Qubit Erasure Error:\n"
        rst += f"  {alert if node['error_model']['pe'] > 0 else ' '}     pE: {node['error_model']['pe']}"
        tensor = f"\u00D7"
        if node['gate_peer'] is not None:
            rst += "\n"
            gate_peer = Position(node['gate_peer'])
            rst += f"    Peer Qubit of this Two-Qubit Gate: [{gate_peer.t}][{gate_peer.i}][{gate_peer.j}]:\n"
            rst += f"      Correlated Two-Qubit Pauli Error: (this {tensor} peer)\n"
            correlated_error_model = node['error_model']['corr_pp']
            rst += f"  {alert if correlated_error_model['pix'] > 0 else ' '}     pIX = {correlated_error_model['pix']}\n"
            rst += f"  {alert if correlated_error_model['piz'] > 0 else ' '}     pIZ = {correlated_error_model['piz']}\n"
            rst += f"  {alert if correlated_error_model['piy'] > 0 else ' '}     pIY = {correlated_error_model['piy']}\n"
            rst += f"  {alert if correlated_error_model['pxi'] > 0 else ' '}     pXI = {correlated_error_model['pxi']}\n"
            rst += f"  {alert if correlated_error_model['pxx'] > 0 else ' '}     pXX = {correlated_error_model['pxx']}\n"
            rst += f"  {alert if correlated_error_model['pxz'] > 0 else ' '}     pXZ = {correlated_error_model['pxz']}\n"
            rst += f"  {alert if correlated_error_model['pxy'] > 0 else ' '}     pXY = {correlated_error_model['pxy']}\n"
            rst += f"  {alert if correlated_error_model['pzi'] > 0 else ' '}     pZI = {correlated_error_model['pzi']}\n"
            rst += f"  {alert if correlated_error_model['pzx'] > 0 else ' '}     pZX = {correlated_error_model['pzx']}\n"
            rst += f"  {alert if correlated_error_model['pzz'] > 0 else ' '}     pZZ = {correlated_error_model['pzz']}\n"
            rst += f"  {alert if correlated_error_model['pzy'] > 0 else ' '}     pZY = {correlated_error_model['pzy']}\n"
            rst += f"  {alert if correlated_error_model['pyi'] > 0 else ' '}     pYI = {correlated_error_model['pyi']}\n"
            rst += f"  {alert if correlated_error_model['pyx'] > 0 else ' '}     pYX = {correlated_error_model['pyx']}\n"
            rst += f"  {alert if correlated_error_model['pyz'] > 0 else ' '}     pYZ = {correlated_error_model['pyz']}\n"
            rst += f"  {alert if correlated_error_model['pyy'] > 0 else ' '}     pYY = {correlated_error_model['pyy']}\n"
            correlated_sum = 0
            for name in correlated_error_model:
                correlated_sum += correlated_error_model[name]
            rst += f"  {alert if correlated_sum > 0 else ' '}       => sum = {correlated_sum}\n"
            rst += f"      Correlated Two-Qubit Erasure Error: (this {tensor} peer)\n"
            correlated_erasure_error_model = node['error_model']['corr_pe']
            rst += f"  {alert if correlated_erasure_error_model['pie'] > 0 else ' '}     pIE = {correlated_erasure_error_model['pie']}\n"
            rst += f"  {alert if correlated_erasure_error_model['pei'] > 0 else ' '}     pEI = {correlated_erasure_error_model['pei']}\n"
            rst += f"  {alert if correlated_erasure_error_model['pee'] > 0 else ' '}     pEE = {correlated_erasure_error_model['pee']}\n"
            correlated_erasure_sum = 0.
            for name in correlated_erasure_error_model:
                correlated_erasure_sum += correlated_erasure_error_model[name]
            rst += f"  {alert if correlated_erasure_sum > 0 else ' '}       => sum = {correlated_erasure_sum}"
        return rst
    def _get_pX(self):
        return self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]['error_model']['pp']['px']
    def _set_pX(self, error_rate):
        self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]['error_model']['pp']['px'] = error_rate
    pX = property(
        fset=_set_pX,
        fget=_get_pX,
        doc="single-qubit error rate of Pauli X"
    )

    def _get_pZ(self):
        return self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]['error_model']['pp']['pz']
    def _set_pZ(self, error_rate):
        self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]['error_model']['pp']['pz'] = error_rate
    pZ = property(
        fset=_set_pZ,
        fget=_get_pZ,
        doc="single-qubit error rate of Pauli Z"
    )

    def _get_pY(self):
        return self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]['error_model']['pp']['py']
    def _set_pY(self, error_rate):
        self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]['error_model']['pp']['py'] = error_rate
    pY = property(
        fset=_set_pY,
        fget=_get_pY,
        doc="single-qubit error rate of Pauli Y"
    )

    def _get_pE(self):
        return self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]['error_model']['erasure_error_rate']
    def _set_pE(self, error_rate):
        self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]['error_model']['erasure_error_rate'] = error_rate
    pE = property(
        fset=_set_pE,
        fget=_get_pE,
        doc="single-qubit error rate of erasure (i.e. detected random Pauli error at known position)"
    )

    # correlated error model
    def _get_pIX(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of IX is invalid")
        return node['error_model']['corr_pp']['error_rate_ix']
    def _set_pIX(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of IX is invalid")
        node['error_model']['corr_pp']['error_rate_ix'] = error_rate
    pIX = property(
        fset=_set_pIX,
        fget=_get_pIX,
        doc="two-qubit error rate of Pauli IX"
    )

    def _get_pIZ(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of IZ is invalid")
        return node['error_model']['corr_pp']['error_rate_iz']
    def _set_pIZ(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of IZ is invalid")
        node['error_model']['corr_pp']['error_rate_iz'] = error_rate
    pIZ = property(
        fset=_set_pIZ,
        fget=_get_pIZ,
        doc="two-qubit error rate of Pauli IZ"
    )

    def _get_pIY(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of IY is invalid")
        return node['error_model']['corr_pp']['error_rate_iy']
    def _set_pIY(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of IY is invalid")
        node['error_model']['corr_pp']['error_rate_iy'] = error_rate
    pIY = property(
        fset=_set_pIY,
        fget=_get_pIY,
        doc="two-qubit error rate of Pauli IY"
    )

    def _get_pXI(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of XI is invalid")
        return node['error_model']['corr_pp']['error_rate_xi']
    def _set_pXI(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of XI is invalid")
        node['error_model']['corr_pp']['error_rate_xi'] = error_rate
    pXI = property(
        fset=_set_pXI,
        fget=_get_pXI,
        doc="two-qubit error rate of Pauli XI"
    )

    def _get_pXX(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of XX is invalid")
        return node['error_model']['corr_pp']['error_rate_xx']
    def _set_pXX(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of XX is invalid")
        node['error_model']['corr_pp']['error_rate_xx'] = error_rate
    pXX = property(
        fset=_set_pXX,
        fget=_get_pXX,
        doc="two-qubit error rate of Pauli XX"
    )

    def _get_pXZ(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of XZ is invalid")
        return node['error_model']['corr_pp']['error_rate_xz']
    def _set_pXZ(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of XZ is invalid")
        node['error_model']['corr_pp']['error_rate_xz'] = error_rate
    pXZ = property(
        fset=_set_pXZ,
        fget=_get_pXZ,
        doc="two-qubit error rate of Pauli XZ"
    )

    def _get_pXY(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of XY is invalid")
        return node['error_model']['corr_pp']['error_rate_xy']
    def _set_pXY(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of XY is invalid")
        node['error_model']['corr_pp']['error_rate_xy'] = error_rate
    pXY = property(
        fset=_set_pXY,
        fget=_get_pXY,
        doc="two-qubit error rate of Pauli XY"
    )

    def _get_pZI(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of ZI is invalid")
        return node['error_model']['corr_pp']['error_rate_zi']
    def _set_pZI(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of ZI is invalid")
        node['error_model']['corr_pp']['error_rate_zi'] = error_rate
    pZI = property(
        fset=_set_pZI,
        fget=_get_pZI,
        doc="two-qubit error rate of Pauli ZI"
    )

    def _get_pZX(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of ZX is invalid")
        return node['error_model']['corr_pp']['error_rate_zx']
    def _set_pZX(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of ZX is invalid")
        node['error_model']['corr_pp']['error_rate_zx'] = error_rate
    pZX = property(
        fset=_set_pZX,
        fget=_get_pZX,
        doc="two-qubit error rate of Pauli ZX"
    )

    def _get_pZZ(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of ZZ is invalid")
        return node['error_model']['corr_pp']['error_rate_zz']
    def _set_pZZ(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of ZZ is invalid")
        node['error_model']['corr_pp']['error_rate_zz'] = error_rate
    pZZ = property(
        fset=_set_pZZ,
        fget=_get_pZZ,
        doc="two-qubit error rate of Pauli ZZ"
    )

    def _get_pZY(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of ZY is invalid")
        return node['error_model']['corr_pp']['error_rate_zy']
    def _set_pZY(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of ZY is invalid")
        node['error_model']['corr_pp']['error_rate_zy'] = error_rate
    pZY = property(
        fset=_set_pZY,
        fget=_get_pZY,
        doc="two-qubit error rate of Pauli ZY"
    )

    def _get_pYI(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of YI is invalid")
        return node['error_model']['corr_pp']['error_rate_yi']
    def _set_pYI(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of YI is invalid")
        node['error_model']['corr_pp']['error_rate_yi'] = error_rate
    pYI = property(
        fset=_set_pYI,
        fget=_get_pYI,
        doc="two-qubit error rate of Pauli YI"
    )

    def _get_pYX(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of YX is invalid")
        return node['error_model']['corr_pp']['error_rate_yx']
    def _set_pYX(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of YX is invalid")
        node['error_model']['corr_pp']['error_rate_yx'] = error_rate
    pYX = property(
        fset=_set_pYX,
        fget=_get_pYX,
        doc="two-qubit error rate of Pauli YX"
    )

    def _get_pYZ(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of YZ is invalid")
        return node['error_model']['corr_pp']['error_rate_yz']
    def _set_pYZ(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of YZ is invalid")
        node['error_model']['corr_pp']['error_rate_yz'] = error_rate
    pYZ = property(
        fset=_set_pYZ,
        fget=_get_pYZ,
        doc="two-qubit error rate of Pauli YZ"
    )

    def _get_pYY(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of YY is invalid")
        return node['error_model']['corr_pp']['error_rate_yy']
    def _set_pYY(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of YY is invalid")
        node['error_model']['corr_pp']['error_rate_yy'] = error_rate
    pYY = property(
        fset=_set_pYY,
        fget=_get_pYY,
        doc="two-qubit error rate of Pauli YY"
    )

    def _get_pIE(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of IE is invalid")
        return node['error_model']['corr_pe']['error_rate_ie']
    def _set_pIE(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of IE is invalid")
        node['error_model']['corr_pe']['error_rate_ie'] = error_rate
    pIE = property(
        fset=_set_pIE,
        fget=_get_pIE,
        doc="two-qubit error rate of Pauli IE"
    )

    def _get_pEI(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of EI is invalid")
        return node['error_model']['corr_pe']['error_rate_ei']
    def _set_pEI(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of EI is invalid")
        node['error_model']['corr_pe']['error_rate_ei'] = error_rate
    pEI = property(
        fset=_set_pEI,
        fget=_get_pEI,
        doc="two-qubit error rate of Pauli EI"
    )

    def _get_pEE(self):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of EE is invalid")
        return node['error_model']['corr_pe']['error_rate_ee']
    def _set_pEE(self, error_rate):
        node = self.source.error_model["nodes"][self.position.t][self.position.i][self.position.j]
        if node["gate_peer"] is None:
            raise Exception(f"qubit[{self.position.t}][{self.position.i}][{self.position.j}] doesn't have two-qubit gate here, error rate of EE is invalid")
        node['error_model']['corr_pe']['error_rate_ee'] = error_rate
    pEE = property(
        fset=_set_pEE,
        fget=_get_pEE,
        doc="two-qubit error rate of Pauli EE"
    )

class ErrorModelName:
    def __init__(self, error_model, di, dj, measurement_rounds, p, pe, configuration):
        self.error_model = error_model
        self.di = di
        self.dj = dj
        self.measurement_rounds = measurement_rounds
        self.p = p
        self.pe = pe
        self.configuration = configuration
    def at(self, position):
        nodes = self.error_model["nodes"]
        if position.t < len(nodes) and nodes[position.t] is not None:
            nodes_row_0 = nodes[position.t]
            if position.i < len(nodes_row_0) and nodes_row_0[position.i] is not None:
                nodes_row_1 = nodes_row_0[position.i]
                if position.j < len(nodes_row_1) and nodes_row_1[position.j] is not None:
                    # node = nodes_row_1[j]
                    return QubitErrorModel(self, position)
        # raise Exception(f"position [{t}][{i}][{j}] not exist")
        return None

    def _get_height(self):
        return self.error_model["height"]
    height = property(
        fget=_get_height,
        doc="0 <= t <= height"
    )

    def _get_vertical(self):
        return self.error_model["vertical"]
    vertical = property(
        fget=_get_vertical,
        doc="0 <= i <= vertical"
    )

    def _get_horizontal(self):
        return self.error_model["horizontal"]
    horizontal = property(
        fget=_get_horizontal,
        doc="0 <= j <= horizontal"
    )

    def visualize(self, api_base_url="https://qec.wuyue98.cn/api", viewer_url="https://qec.wuyue98.cn/ErrorModelViewer2D.html"):
        assert self.di == 5 and self.dj == 5 and self.measurement_rounds == 5, "visualization tool currently only support di = dj = measurement_rounds = 5"
        # upload to temporary store
        modified_error_model = json.dumps(self.error_model)
        data = json.dumps({"value": modified_error_model}).encode('utf-8')
        req = request.Request(api_base_url + "/new_temporary_store", method="POST", data=data)
        req.add_header('Content-Type', 'application/json')
        try:
            response = request.urlopen(req)
            error_model_temporary_id = response.read().decode('utf-8')
        except HTTPError as e:
            print(e)
            raise e
        except URLError as e:
            print('Reason: ', e.reason)
            raise e
        except Exception as e:
            raise e
        print()
        # generate viewer link
        configuration_escaped = parse.quote(" ".join(self.configuration).encode('utf8'))
        query_paramters = f"?p={float(self.p)}&pe={float(self.pe)}&parameters={configuration_escaped}&error_model_temporary_id={error_model_temporary_id}"
        if viewer_url is not None:
            url = viewer_url + query_paramters
            webbrowser.open(url, new=2)
        else:
            print(f"viewer_url not provided, please open html viewer (e.g. QEC-Playground/frontend/ErrorModelViewer2D.html) and append the following parameters to it:")
            print(f"    {query_paramters}")
        download_url = api_base_url + "/get_temporary_store/" + error_model_temporary_id
        print(f"[info] you can download the error model Json file using: {download_url}")
        return error_model_temporary_id

    def save(self, filepath):
        with open(filepath, "w", encoding="utf8") as f:
            modified_error_model = json.dumps(self.error_model)
            f.write(modified_error_model)
            f.flush()
    
    def run_benchmark(self, max_N=100000, min_error_cases=3000, time_budget=None, verbose=False):
        delete_tmp_file = True
        if verbose:
            delete_tmp_file = False  # user expect to run the printed command later
        out_file = tempfile.NamedTemporaryFile(delete=delete_tmp_file)  # this file will be deleted as long as the file is closed if delete=True
        out_filename = out_file.name
        with open(out_filename, "w", encoding="utf8") as f:
            modified_error_model = json.dumps(self.error_model)
            f.write(modified_error_model)
            f.flush()
            command = qec_playground_benchmark_simulator_runner_vec_command([self.p], [self.di], [self.dj], [self.measurement_rounds], ["--pes", f"[{self.pe}]"] + self.configuration + ["--load_error_model_from_file"
                , f"{out_filename}"], max_N=max_N, min_error_cases=min_error_cases, time_budget=time_budget)
            if verbose:
                print("[verbose] command:", " ".join(command))
            stdout, returncode = run_qec_playground_command_get_stdout(command)
        # print("\n" + stdout)
        # assert returncode == 0, "command fails..."
        return stdout, returncode

def fetch_error_model(di, dj, noisy_measurement_rounds, p=0, pe=0, configuration=[]):
    qecp_path = os.path.join(rust_dir, "target", "release", "qecp")
    command = [qecp_path, "tool", "benchmark"] + [f"[{di}]", "--djs", f"[{dj}]", f"[{noisy_measurement_rounds}]", f"[{p}]", "--pes", f"[{pe}]"]  + configuration + ["--debug_print", "full-error-model"]
    stdout, returncode = run_qec_playground_command_get_stdout(command)
    if returncode != 0:
        print("[error] failed command:", " ".join(command))
        exit(1)
    json_string = stdout.strip(" \r\n").split("\n")[-1]
    # print(json_string)
    return ErrorModelName(json.loads(json_string), di, dj, noisy_measurement_rounds, p, pe, configuration)
