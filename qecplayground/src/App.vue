<template>
	<div id="app">
		<MainQubits class="main-qubits" ref="qubits" :panelWidth="480" :enableStats="enableStats" :decoderServerRootUrl="decoderServerRootUrl"
			:L="L" @dataQubitClicked="dataQubitClicked"></MainQubits>
		<div class="control-panel">
			<div style="text-align: center;">
				<h1 class="title"><img src="@/assets/logo.png" class="logo"/>QEC Playground</h1>
				<p>This is an educational tool for Quantum Error Correction (QEC). You can learn the currently most promising QEC scheme called surface code (planar code) by following the introduction tutorial and then trying different error patterns interactively.</p>
			</div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Global Settings</span>
					<el-button style="float: right; padding: 3px 0" type="text" disabled>help</el-button>
				</div>
				<div style="position: relative;">
					Code Distance:
					<el-input-number v-model="code_distance" @change="code_distance_changed"></el-input-number>
					<div style="height: 20px;"></div>
					Pauli Operators:
					<el-radio-group v-model="pauli_display">
						<el-radio-button label="error_only">Error Only</el-radio-button>
						<el-radio-button label="correction_only">Correction</el-radio-button>
						<el-radio-button label="both">Both</el-radio-button>
					</el-radio-group>
					<div style="height: 20px;"></div>
					<el-switch v-model="measurement_display" active-text="display measurement" inactive-text="Do not display measuremnt"></el-switch>
				</div>
			</el-card>
			<div style="height: 10px;"></div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Customize Error Pattern</span>
					<el-button style="float: right; padding: 3px 0" type="text" disabled>help</el-button>
				</div>
				<div style="position: relative;">
					<el-button type="success" class="toggle-error-button" :plain="!toggle_X_error" @click="enable_toggle_error(true)">
						Toggle X Error (bit-flip error)</el-button>
					<div style="height: 10px;"></div>
					<el-button type="primary" class="toggle-error-button" :plain="!toggle_Z_error" @click="enable_toggle_error(false)">
						Toggle Z Error (phase-flip error)</el-button>
					<el-button type="danger" class="clear-error-button" @click="clear_error()" :disabled="pauli_display!='error_only'">Clear Error</el-button>
				</div>
			</el-card>
			<div style="height: 10px;"></div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Error Correction</span>
					<el-button style="float: right; padding: 3px 0" type="text" disabled>help</el-button>
				</div>
				<div style="position: relative;">
					Select Decoder:
					<el-select v-model="decoder" placeholder="Select a decoder">
						<el-option v-for="item in available_decoders" :key="item.value" :label="item.label" :value="item.value"> </el-option>
					</el-select>
					<div style="height: 15px;"></div>
					<el-button type="success" class="big-button" @click="run_correction">Run Correction</el-button>
				</div>
			</el-card>
		</div>
	</div>
</template>

<script>
import MainQubits from './components/MainQubits.vue'
let deploy_mode = process.env.NODE_ENV != "development"

export default {
	name: 'app',
	components: {
		MainQubits,
	},
	data() {
		return {
			deploy_mode: deploy_mode,

			toggle_X_error: false,
			toggle_Z_error: false,
			pauli_display: "error_only",
			measurement_display: true,
			code_distance: 5,
			L: 5,
			decoder: "stupid_decoder",
			available_decoders: [
				{ value: "stupid_decoder", label: "Stupid Decoder" },
			],

			x_error: [ ],  // [L][L] 0 ~ 1
			z_error: [ ],  // [L][L] 0 ~ 1
			x_correction: [ ],  // [L][L] 0 ~ 1
			z_correction: [ ],  // [L][L] 0 ~ 1
			measurement: [ ],  // [L+1][L+1] 0 ~ 1
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
		this.onChangeL()
	},
	methods: {
		copy_matrix(target, source) {
			for (let i=0; i<source.length; ++i) {
				for (let j=0; j<source[i].length; ++j) {
					target[i][j] = source[i][j]
				}
			}
		},
		refresh() {
			if (this.pauli_display == "error_only") {
				this.copy_matrix(this.$refs.qubits.xDataQubitsErrors, this.x_error)
				this.copy_matrix(this.$refs.qubits.zDataQubitsErrors, this.z_error)
				this.$refs.qubits.update_measurement()
			}
		},
		dataQubitClicked(data) {
			let [i, j, absTime] = data
			if (!this.toggle_X_error && !this.toggle_Z_error) return  // ignore event
			if (this.pauli_display != "error_only") {
				this.$notify.error({
					title: 'Action Failed',
					message: 'You can only customize error pattern in "Error Only" mode (see "Global Settings" -> "Pauli Operators")'
				})
				return
			}
			if (this.toggle_X_error) {
				this.x_error[i][j] = 1 - this.x_error[i][j]
			}
			if (this.toggle_Z_error) {
				this.z_error[i][j] = 1 - this.z_error[i][j]
			}
			this.refresh()
		},
		enable_toggle_error(is_X) {
			if (is_X) {
				this.toggle_X_error = !this.toggle_X_error
			} else {
				this.toggle_Z_error = !this.toggle_Z_error
			}
		},
		clear_error() {
			for (let i=0; i < this.L; ++i) {
				for (let j=0; j < this.L; ++j) {
					this.x_error[i][j] = 0
					this.z_error[i][j] = 0
				}
			}
			this.refresh()
		},
		code_distance_changed(val, oldVal) {
			if (val < oldVal) this.code_distance = Math.floor((val - 1) / 2) * 2 + 1
			else this.code_distance = Math.ceil((val - 1) / 2) * 2 + 1
			this.L = this.code_distance
		},
		async run_correction() {
			let data = await this.$refs.qubits.get_correction(this.decoder)
			console.log(data)
		},
		onChangeL() {
			this.x_error = this.$refs.qubits.makeSquareArray(this.L)
			this.z_error = this.$refs.qubits.makeSquareArray(this.L)
			this.x_correction = this.$refs.qubits.makeSquareArray(this.L)
			this.z_correction = this.$refs.qubits.makeSquareArray(this.L)
			this.measurement = this.$refs.qubits.makeSquareArray(this.L + 1)
		},
	},
	watch: {
		L() {
			this.onChangeL()
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

.big-button {
	width: 100%;
	height: 100px;
}

</style>
