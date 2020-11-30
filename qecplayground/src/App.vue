<template>
	<div id="app">
		<MainQubits class="main-qubits" ref="qubits" :panelWidth="480" :enableStats="enableStats" :decoderServerRootUrl="decoderServerRootUrl"
			:toggleXError="toggle_X_error" :toggleZError="toggle_Z_error" :L="checked_code_distance"></MainQubits>
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
						<el-radio-button label="eo">Error Only</el-radio-button>
						<el-radio-button label="co">Correction</el-radio-button>
						<el-radio-button label="bo">Both</el-radio-button>
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
					<el-button type="danger" class="clear-error-button" @click="clear_error()">Clear Error</el-button>
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
			pauli_display: "bo",
			measurement_display: true,
			code_distance: 5,
			checked_code_distance: 5,
			decoder: "stupid_decoder",
			available_decoders: [
				{ value: "stupid_decoder", label: "Stupid Decoder" },
			]
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
		
	},
	methods: {
		enable_toggle_error(is_X) {
			if (is_X) {
				this.toggle_X_error = !this.toggle_X_error
			} else {
				this.toggle_Z_error = !this.toggle_Z_error
			}
		},
		clear_error() {
			this.$refs.qubits.clear_error()
		},
		code_distance_changed(val, oldVal) {
			if (val < oldVal) this.code_distance = Math.floor((val - 1) / 2) * 2 + 1
			else this.code_distance = Math.ceil((val - 1) / 2) * 2 + 1
			this.checked_code_distance = this.code_distance
		},
		async run_correction() {
			let data = await this.$refs.qubits.get_correction(this.decoder)
			console.log(data)
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
