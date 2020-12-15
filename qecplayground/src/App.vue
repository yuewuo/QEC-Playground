<template>
	<div id="app">
		<FaultTolerantView class="main-qubits" :panelWidth="480" :L="L" :T="T" :showDataQubit="show_data_qubit" :showXAncilla="show_X_ancilla"
			:showZAncilla="show_Z_ancilla" :showVerticalLine="show_vertical_line" :showInitialization="show_initialization" :showCXGates="show_CX_gates"
			:showXEdges="show_X_edges" :showZEdges="show_Z_edges" :useRotated="use_rotated" :depolarErrorRate="0.001" ref="ft_view"
			:usePerspectiveCamera="use_perspective_camera"></FaultTolerantView>
		<div class="control-panel no-scrollbar">
			<div style="text-align: center;">
				<h1 class="title"><img src="@/assets/logo.png" class="logo"/>QEC Playground</h1>
				<p>This page is a visualization tool of fault-tolerant surface code. For tutorial and simpler case, visit <a href="https://wuyue98.cn/QECPlayground/" target="_blank">QECPlayground</a> instead.</p>
			</div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Display Settings</span>
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
					<el-switch v-model="use_perspective_camera" active-text="Perspective Camera" inactive-text="Orthogonal Camera"></el-switch>
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
			<div style="height: 10px;"></div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Random Error Generator (I + p<sub>X</sub>X + p<sub>Z</sub>Z + p<sub>Y</sub>Y)</span>
				</div>
				<div style="position: relative">
					<div>
						<div class="probability">p<sub>X</sub></div>
						<el-input style="width: 90px;" v-model="error_rate_x" placeholder="0"></el-input>
						<div class="probability">p<sub>Z</sub></div>
						<el-input style="width: 90px;" v-model="error_rate_z" placeholder="0"></el-input>
						<div class="probability">p<sub>Y</sub></div>
						<el-input style="width: 90px;" v-model="error_rate_y" placeholder="0"></el-input>
					</div>
					<div style="height: 20px;"></div>
					<el-button type="success" style="width: 100%" @click="generate_random_error">Generate i.i.d. Random Errors</el-button>
				</div>
			</el-card>
			<div style="height: 10px;"></div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Customize Error</span>
				</div>
				<div style="position: relative">
					<el-button type="warning" style="width: 100%" @click="clear_error">Clear All Errors</el-button>
					<div style="height: 20px;"></div>
					<div>
						<div v-for="(item, index) of error_info" v-bind:key="index" :style="{ color: item[4] }">
							{{ `${item[3]} error at t=${item[0]}, i=${item[1]}, j=${item[2]}` }}
						</div>
					</div>
					<div style="height: 20px;" v-if="error_info.length > 0"></div>
					<div>
						<div class="index">t</div>
						<el-input-number v-model="target_t" controls-position="right" :min="0" :max="6*T" size="medium"></el-input-number>
						<div class="index">i</div>
						<el-input-number v-model="target_i" controls-position="right" :min="0" :max="2*L" size="medium"></el-input-number>
						<div class="index">j</div>
						<el-input-number v-model="target_j" controls-position="right" :min="0" :max="2*L" size="medium"></el-input-number>
					</div>
					<div style="height: 20px;"></div>
					<div>
						<el-button type="info" class="set-error-button" @click="set_error(false, false)">I</el-button>
						<el-button type="success" class="set-error-button" @click="set_error(true, false)">X</el-button>
						<el-button type="primary" class="set-error-button" @click="set_error(false, true)">Z</el-button>
						<el-button type="danger" class="set-error-button" @click="set_error(true, true)">Y</el-button>
					</div>
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
			use_perspective_camera: true,
			
			target_t: 0,
			target_i: 0,
			target_j: 0,

			error_rate_x: "1e-3",
			error_rate_z: "1e-3",
			error_rate_y: "1e-3",

			error_info: [],  // [t, i, j, name, color]
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
		set_error(X, Z) {
			const ft_view = this.$refs.ft_view
			const node = ft_view.get_snapshot_node(this.target_t, this.target_i, this.target_j)
			if (node) {
				node.error = X ? (Z ? ft_view.constants.ETYPE.Y : ft_view.constants.ETYPE.X) : (Z ? ft_view.constants.ETYPE.Z : ft_view.constants.ETYPE.I)
				ft_view.compute_propagated_error()
				this.update_error_information()
			} else {
				this.$notify.error({
					title: 'Set Error Failed',
					message: `Node at [${this.target_t}][${this.target_i}][${this.target_j}] doesn't exist.`
				})
			}
		},
		clear_error() {
			let ft_view = this.$refs.ft_view
			ft_view.iterate_snapshot((node, t, i, j) => {
				node.error = ft_view.constants.ETYPE.I
			})
			ft_view.compute_propagated_error()
			this.update_error_information()
		},
		generate_random_error() {
			let ft_view = this.$refs.ft_view
			const [error_rate_x, error_rate_z, error_rate_y] = this.get_error_rates()
			ft_view.iterate_snapshot((node, t, i, j) => {
				node.error_rate_x = error_rate_x
				node.error_rate_z = error_rate_z
				node.error_rate_y = error_rate_y
			})
			ft_view.generate_random_error()
			ft_view.compute_propagated_error()
			this.update_error_information()
		},
		get_error_rates() {
			function fixed_error_rate(that, name) {
				const error_rate = parseFloat(that[name])
				if (!(error_rate <= 1 && error_rate >= 0)) {
					that[name] = "0"
					return 0
				}
				return error_rate
			}
			const error_rate_x = fixed_error_rate(this, "error_rate_x")
			const error_rate_z = fixed_error_rate(this, "error_rate_z")
			const error_rate_y = fixed_error_rate(this, "error_rate_y")
			if (error_rate_x + error_rate_z + error_rate_y > 1) {
				this.$notify.error({
					title: 'Invalid Error Rate',
					message: `The sum of error rate probabilities should not exceed 1, now it's ${error_rate_x + error_rate_z + error_rate_y}.`
				})
				return [0, 0, 0]
			}
			return [error_rate_x, error_rate_z, error_rate_y]
		},
		update_error_information() {
			let ft_view = this.$refs.ft_view
			let error_info = []
			ft_view.iterate_snapshot((node, t, i, j) => {
				if (node.error != ft_view.constants.ETYPE.I) {
					let name = "I"
					let color = "black"
					if (node.error == ft_view.constants.ETYPE.X) { name = "X"; color = "#85ce61" }
					if (node.error == ft_view.constants.ETYPE.Z) { name = "Z"; color = "#409EFF" }
					if (node.error == ft_view.constants.ETYPE.Y) { name = "Y"; color = "#F56C6C" }
					error_info.push([t, i, j, name, color])
				}
			})
			this.error_info = error_info
		},
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

/* .no-scrollbar::-webkit-scrollbar {
	width: 10px;
} */

.el-input-number.is-controls-right .el-input__inner {
	padding-left: 0 !important;
	padding-right: 30px !important;
	width: 100px;
}

.el-input-number.is-controls-right .el-input-number__decrease {
	left: 62px !important;
}

.el-input-number.is-controls-right .el-input-number__increase {
	left: 62px !important;
}

.el-input-number.el-input-number--medium.is-controls-right {
	width: 110px !important;
}

.index {
	font-size: 150%;
	font-weight: bold;
	display: inline;
	margin-right: 10px;
}

.probability {
	font-size: 120%;
	display: inline;
	margin-right: 10px;
	margin-left: 10px;
}

.set-error-button {
	width: 90px;
	margin-right: 10px;
}

</style>
