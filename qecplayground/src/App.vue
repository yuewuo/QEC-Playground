<template>
	<div id="app">
		<FaultTolerantView class="main-qubits" :panelWidth="480" :L="L" :T="T" :showDataQubit="show_data_qubit" :showXAncilla="show_X_ancilla"
			:showZAncilla="show_Z_ancilla" :showVerticalLine="show_vertical_line" :showInitialization="show_initialization" :showCXGates="show_CX_gates"
			:showXEdges="show_X_edges" :showZEdges="show_Z_edges" :useRotated="use_rotated"></FaultTolerantView>
		<div class="control-panel no-scrollbar">
			<div style="text-align: center;">
				<h1 class="title"><img src="@/assets/logo.png" class="logo"/>QEC Playground</h1>
				<p>This page is a visualization tool of fault-tolerant surface code. For tutorial and simpler case, visit <a href="https://wuyue98.cn/QECPlayground/" target="_blank">QECPlayground</a> instead.</p>
			</div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Display Settings</span>
					<!-- <el-button style="float: right; padding: 3px 0" type="text" disabled>help</el-button> -->
				</div>
				<div style="position: relative">
					<div>
						Code Distance:
						<el-input-number v-model="bufferedL" :min="3"></el-input-number>
					</div>
					<div style="height: 20px;"></div>
					<div>
						Measurement Round:
						<el-input-number v-model="T" :min="1"></el-input-number>
					</div>
					<div style="height: 20px;"></div>
					<el-switch v-model="use_rotated" active-text="Rotated Planar Code" inactive-text="Standard Planar Code"></el-switch>
					<div style="height: 20px;"></div>
					<el-switch v-model="show_data_qubit" active-text="Show Data Qubits" inactive-text="Hide"></el-switch>
					<div style="height: 20px;"></div>
					<el-switch v-model="show_Z_ancilla" active-text="Show Z Stabilizers" inactive-text="Hide"></el-switch>
					<div style="height: 20px;"></div>
					<el-switch v-model="show_X_ancilla" active-text="Show X Stabilizers" inactive-text="Hide"></el-switch>
					<div style="height: 20px;"></div>
					<el-switch v-model="show_vertical_line" active-text="Show Vertical Lines" inactive-text="Hide"></el-switch>
					<div style="height: 20px;"></div>
					<el-switch v-model="show_initialization" active-text="Show Initialization" inactive-text="Hide"></el-switch>
					<div style="height: 20px;"></div>
					<el-switch v-model="show_CX_gates" active-text="Show CX (CNOT) Gates" inactive-text="Hide"></el-switch>
					<div style="height: 20px;"></div>
					<el-switch v-model="show_Z_edges" active-text="Show Z Graph Lattice" inactive-text="Hide"></el-switch>
					<div style="height: 20px;"></div>
					<el-switch v-model="show_X_edges" active-text="Show X Graph Lattice" inactive-text="Hide"></el-switch>
				</div>
			</el-card>
		</div>
	</div>
</template>

<script>
import FaultTolerantView from './components/FaultTolerantView'
let deploy_mode = process.env.NODE_ENV != "development"

export default {
	name: 'app',
	components: {
		FaultTolerantView,
	},
	data() {
		return {
			deploy_mode: deploy_mode,

			L: 3,
			T: 3,
			use_rotated: false,

			bufferedL: 1,  // to avoid invalid `L` pass into FaultTolerantView
			show_data_qubit: true,
            show_X_ancilla: true,
            show_Z_ancilla: true,
            show_vertical_line: true,
            show_initialization: true,
            show_CX_gates: true,
            show_X_edges: false,
            show_Z_edges: false,
		}
	},
	computed: {
		enableStats() {
			return !deploy_mode
		},
		decoderServerRootUrl() {
			return deploy_mode ? "/api/qecp/" : "http://127.0.0.1:8066/"
		},
	},
	mounted() {
		window.$app = this  // for fast debugging
		this.bufferedL = this.L
	},
	methods: {
		
	},
	watch: {
		bufferedL(val, oldVal) {
			if (val <= 0) {
				this.bufferedL = 1
			} else {
				if (this.use_rotated) {
					// make it odd
					if (val < oldVal) this.bufferedL = Math.floor((val - 1) / 2) * 2 + 1
					else this.bufferedL = Math.ceil((val - 1) / 2) * 2 + 1
				}
			}
			this.L = this.bufferedL
		},
		use_rotated(val) {
			if (val && this.bufferedL % 2 == 0) {
				this.bufferedL = this.bufferedL + 1  // make it odd when switch to rotated planar code
			}
			this.L = this.bufferedL
		},
	},
}
</script>

<style>

#app {
	font-family: 'Avenir', Helvetica, Arial, sans-serif;
	-webkit-font-smoothing: antialiased;
	-moz-osx-font-smoothing: grayscale;
	color: #2c3e50;
	position: fixed;
	top: 0;
	left: 0;
	right: 0;
	bottom: 0;
}

.main-qubits {
	position: fixed;
	top: 0;
	left: 0;
	right: 480px;
	bottom: 0;
}

.control-panel {
	position: fixed;
	top: 0;
	right: 0;
	bottom: 0;
	width: 460px;
	overflow: auto;
	padding: 10px;
}

.title {
	line-height: 48px;
	position: relative;
}

.logo {
	width: 48px;
	height: 48px;
	position: relative;
	top: 10px;
	right: 10px;
}

.toggle-error-button {
	width: 300px;
}

.clear-error-button {
	position: absolute;
	right: 0;
	top: 0;
	bottom: 0;
}

.full-width {
	width: 100%;
}

.big-button {
	width: 100%;
	height: 100px;
}

.no-scrollbar::-webkit-scrollbar {
	width: 0;
}

.el-button--success.is-plain:hover, .el-button--success.is-plain:focus {
    color: #67C23A !important;
    background: #f0f9eb !important;
    border-color: #c2e7b0 !important;
}

.el-button--primary.is-plain:hover, .el-button--primary.is-plain:focus {
    color: #409EFF !important;
    background: #ecf5ff !important;
    border-color: #b3d8ff !important;
}

</style>
