<template>
	<div id="app">
		<MainQubits :removeView="remove_3d_view" class="main-qubits" ref="qubits" :panelWidth="480" :enableStats="enableStats" 
			:decoderServerRootUrl="decoderServerRootUrl" :L="L" @dataQubitClicked="dataQubitClicked" :websiteRoot="websiteRoot"
			:hideZancilla="running != null && hideZancilla" :hideXancilla="running != null && hideXancilla"></MainQubits>
		<div class="control-panel no-scrollbar">
			<div style="text-align: center;">
				<h1 class="title"><img src="@/assets/logo.png" class="logo"/>QEC Playground</h1>
				<p>This is an educational tool for Quantum Error Correction (QEC). You can learn the currently most promising QEC scheme called surface code (planar code) by following the introduction tutorial and then trying different error patterns interactively.</p>
			</div>
			<el-collapse-transition>
				<div v-show="running != null" v-if="tutorial_contents != null">
					<el-card :body-style="{ padding: '20px', background: 'yellow', position: 'relative' }" id="tutorial-contents">
						<div v-for="(list, name, index) of tutorial_contents" v-bind:key="index" v-show="running == name">
							<div v-for="(item, i) in list" v-bind:key="i" v-show="running_idx == i">
								<h3 style="margin: 0;" v-if="item.type == 'text'">{{ item.content }}</h3>
							</div>
						</div>
						<div style="margin-top: 15px;">
							<el-button type="info" plain :disabled="running_idx <= 0" @click="tutorial_last">Last</el-button>
							<el-steps :active="running_idx" finish-status="success" style="width: 200px; position: absolute; bottom: 32px; left: 128px;">
								<el-step v-for="(item, i) in tutorial_contents[running]" v-bind:key="i" title=""></el-step>
							</el-steps>
							<el-button type="info" plain style="float: right;" @click="tutorial_next">
								{{ (running && running_idx >= tutorial_contents[running].length - 1) ? "Quit" : "Next" }}</el-button>
						</div>
					</el-card>
					<div style="height: 10px;"></div>
				</div>
			</el-collapse-transition>
			<div>
				<el-button :type="tutorial_show ? 'danger' : 'success'" class="full-width" @click="toggle_tutorial">
					{{tutorial_show ? "Quit Interactive Tutorial" : "Start Interactive Tutorial"}}</el-button>
				<div style="height: 10px;"></div>
			</div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Global Settings</span>
					<!-- <el-button style="float: right; padding: 3px 0" type="text" disabled>help</el-button> -->
				</div>
				<div style="position: relative;">
					<el-tooltip :disabled="!has_tooltip" effect="dark" placement="left">
						<div slot="content">the size of the surface code, containing d<sup>2</sup> data qubits</div>
						<div>
							Code Distance:
							<el-input-number v-model="code_distance" @change="code_distance_changed"></el-input-number>
						</div>
					</el-tooltip>
					<div style="height: 20px;"></div>
					<el-tooltip :disabled="!has_tooltip" effect="dark" content="change current display to customized error pattern, correction or corrected result" placement="left">
						<div>
							Display:
							<el-radio-group v-model="display_mode">
								<el-tooltip :disabled="!has_tooltip" effect="dark" content="customized error pattern which you can edit" placement="top">
									<el-radio-button label="error_only">Error</el-radio-button>
								</el-tooltip>
								<el-tooltip :disabled="!has_tooltip" effect="dark" content="the error correction pattern returned by the decoder" placement="top">
									<el-radio-button label="correction_only">Correction</el-radio-button>
								</el-tooltip>
								<el-tooltip :disabled="!has_tooltip" effect="dark" content="the combine of the former two, to see if the correction is successful" placement="top">
									<el-radio-button label="corrected">Corrected</el-radio-button>
								</el-tooltip>
							</el-radio-group>
						</div>
					</el-tooltip>
					<div style="height: 20px;"></div>
					<el-switch v-model="has_tooltip" active-text="Display tool tips" inactive-text="Do not display tool tips"></el-switch>
					<!-- <div style="height: 10px;"></div>
					<el-tooltip :disabled="!has_tooltip" effect="dark" content="whether display measurement results. this is helpful in interactive tutorial" placement="left">
						<el-switch v-model="measurement_display" active-text="Display measurement" inactive-text="Do not display measuremnt"></el-switch>
					</el-tooltip> -->
				</div>
			</el-card>
			<div style="height: 10px;"></div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Customize Error Pattern</span>
					<!-- <el-button style="float: right; padding: 3px 0" type="text" disabled>help</el-button> -->
				</div>
				<div style="position: relative; width: 100%">
					<el-tooltip :disabled="!has_tooltip" effect="dark" :content="(toggle_X_error ? 'stop' : 'start') + ' toggling qubit X error on clicking'" placement="left">
						<el-button type="success" class="toggle-error-button" :plain="!toggle_X_error" @click="enable_toggle_error(true)">
							Toggle X Error (bit-flip error)</el-button>
					</el-tooltip>
					<div style="height: 10px;"></div>
					<el-tooltip :disabled="!has_tooltip" effect="dark" :content="(toggle_Z_error ? 'stop' : 'start') + ' toggling qubit Z error on clicking'" placement="left">
						<el-button type="primary" class="toggle-error-button" :plain="!toggle_Z_error" @click="enable_toggle_error(false)">
							Toggle Z Error (phase-flip error)</el-button>
					</el-tooltip>
					<el-tooltip :disabled="!has_tooltip" effect="dark" content="clear all errors" placement="top">
						<el-button type="danger" class="clear-error-button" @click="clear_error()" :disabled="display_mode!='error_only'">Clear Error</el-button>
					</el-tooltip>
				</div>
				<div style="position: relative; margin-top: 10px;">
					<el-tooltip :disabled="!has_tooltip" effect="dark" content="take current visible errors as your customized error pattern, change display mode to 'Error'" placement="left">
						<el-button type="info" class="full-width" @click="use_as_error()">
							Use Current Pauli Operators as Error Syndrome</el-button>
					</el-tooltip>
				</div>
			</el-card>
			<div style="height: 10px;"></div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Run Error Correction</span>
					<!-- <el-button style="float: right; padding: 3px 0" type="text" disabled>help</el-button> -->
				</div>
				<div style="position: relative;">
					Select Decoder:
					<el-select v-model="decoder" placeholder="Select a decoder">
						<el-option v-for="item in available_decoders" :key="item.value" :label="item.label" :value="item.value"> </el-option>
					</el-select>
					<el-tooltip :disabled="!has_tooltip" effect="dark" content="clear the correction result" placement="top">
						<el-button type="danger" style="width: 75px; margin-left: 5px;" @click="clear_correction">Clear</el-button>
					</el-tooltip>
					<div style="height: 15px;"></div>
					<div v-if="has_correction">
						<el-alert :title="correction_succeed ? 'Error correction succeeds without breaking the logical state' : 'Error correction fails because ' + correction_fail_reason" :type="correction_succeed ? 'success' : 'error'" :closable="false" show-icon></el-alert>
						<div style="height: 15px;"></div>
					</div>
					<el-tooltip :disabled="!has_tooltip" effect="dark" content="run decoder from remote server" placement="left">
						<el-button type="success" class="big-button" @click="run_correction" :disabled="L < 3">Run Correction</el-button>
					</el-tooltip>
				</div>
			</el-card>
		</div>
		<Tutorial ref="tutorial" :show="tutorial_show" @showing="tutorial_show = $event" @running="running = $event"
			@running_idx="running_idx = $event" @L="tutorial_on_change_L" @hideZancilla="hideZancilla = $event"
			@hideXancilla="hideXancilla = $event" @set_errors="tutorial_set_errors"></Tutorial>
	</div>
</template>

<script>
import MainQubits from './components/MainQubits.vue'
import Tutorial from './components/Tutorial.vue'
let deploy_mode = process.env.NODE_ENV != "development"

export default {
	name: 'app',
	components: {
		MainQubits,
		Tutorial,
	},
	data() {
		return {
			deploy_mode: deploy_mode,

			toggle_X_error: false,
			toggle_Z_error: false,
			display_mode: "error_only",
			modes: {
				error_only: "error_only",
				correction_only: "correction_only",
				corrected: "corrected",
			},
			measurement_display: true,
			max_code_distance: 11,  // otherwise it's too large to render and manipulate
			code_distance: 5,
			L: 5,
			decoder: "MWPM_decoder",
			available_decoders: [
				{ value: "MWPM_decoder", label: "MWPM Decoder" },
				{ value: "naive_decoder", label: "Naive Decoder" },
			],
			has_correction: false,
			correction_succeed: false,
			correction_fail_reason: "some very very long long long reason",

			x_error: [ ],  // [L][L] 0 ~ 1
			z_error: [ ],  // [L][L] 0 ~ 1
			x_correction: [ ],  // [L][L] 0 ~ 1
			z_correction: [ ],  // [L][L] 0 ~ 1
			measurement: [ ],  // [L+1][L+1] 0 ~ 1

			// tutorial related
			tutorial_show: false,
			remove_3d_view: false,
			has_tooltip: deploy_mode ? true : false,  // close tool tips by default when developing
			running: null,
			running_idx: 0,
			tutorial_contents: null,  // `mounted` will copy data from Tutorial.vue
			hideZancilla: false,
			hideXancilla: false,
		}
	},
	computed: {
		enableStats() {
			return !deploy_mode
		},
		decoderServerRootUrl() {
			return deploy_mode ? "/api/qecp/" : "http://127.0.0.1:8066/"
		},
		websiteRoot() {
			return deploy_mode ? "/QECPlayground" : ""  // like https://wuyue98.cn/QECPlayground or https://yuewuo.github.io/QECPlayground
		},
	},
	mounted() {
		window.$app = this  // for fast debugging
		this.onChangeL()
		this.tutorial_contents = this.$refs.tutorial.contents
		let that = this
		this.$nextTick(() => { that.MathjaxConfig.MathQueue("tutorial-contents") })
	},
	methods: {
		clear_correction() {
			this.x_correction = this.$refs.qubits.makeSquareArray(this.L)
			this.z_correction = this.$refs.qubits.makeSquareArray(this.L)
			this.has_correction = false
		},
		copy_matrix(target, source) {
			for (let i=0; i<source.length; ++i) {
				for (let j=0; j<source[i].length; ++j) {
					target[i][j] = source[i][j]
				}
			}
		},
		combined_L2(a, b) {  // a and b should both be [L][L] of 0 ~ 1
			return this.$refs.qubits.makeSquareArray(this.L, (i,j) => a[i][j] ^ b[i][j])
		},
		get_displayed_errors() {
			if (this.display_mode == this.modes.error_only) {
				return [this.x_error, this.z_error]
			}
			if (this.display_mode == this.modes.correction_only) {
				return [this.x_correction, this.z_correction]
			}
			if (this.display_mode == this.modes.corrected) {
				return [this.combined_L2(this.x_error, this.x_correction), this.combined_L2(this.z_error, this.z_correction)]
			}
			throw "unknown display mode"
		},
		refresh() {
			let [display_x_error, display_z_error] = this.get_displayed_errors()
			this.copy_matrix(this.$refs.qubits.xDataQubitsErrors, display_x_error)
			this.copy_matrix(this.$refs.qubits.zDataQubitsErrors, display_z_error)
			if (this.measurement_display) this.$refs.qubits.update_measurement()
			this.$refs.tutorial.on_data_qubit_changed(this.x_error, this.z_error, this.$refs.qubits.ancillaQubitsErrors)
		},
		dataQubitClicked(data) {
			let [i, j, absTime] = data
			if (!this.toggle_X_error && !this.toggle_Z_error) return  // ignore event
			if (this.display_mode != this.modes.error_only) {
				this.$notify.error({
					title: 'Action Failed',
					message: 'You can only customize error pattern in "Error" mode (see "Global Settings" -> "Display")'
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
		use_as_error() {
			let [display_x_error, display_z_error] = this.get_displayed_errors()
			this.x_error = this.$refs.qubits.copySquareArray(display_x_error)
			this.z_error = this.$refs.qubits.copySquareArray(display_z_error)
			this.display_mode = this.modes.error_only
			this.refresh()
		},
		code_distance_changed(val, oldVal) {
			if (val < oldVal) this.code_distance = Math.floor((val - 1) / 2) * 2 + 1
			else this.code_distance = Math.ceil((val - 1) / 2) * 2 + 1
			if (this.code_distance < 1) this.code_distance = 1  // 1 is just for demonstration
			if (this.code_distance > this.max_code_distance) this.code_distance = this.max_code_distance
			if (this.L != this.code_distance) this.L = this.code_distance
		},
		async run_correction() {
			let data = await this.$refs.qubits.get_correction(this.decoder, this.x_error, this.z_error)
			// console.log(data)
			this.x_correction = data.x_correction
			this.z_correction = data.z_correction
			this.display_mode = this.modes.corrected
			this.has_correction = true
			this.correction_succeed = data.x_valid && data.z_valid
			if (!this.correction_succeed) {  // get some reason
				let reason = null
				if (!data.x_valid) {
					if (!data.if_all_z_stabilizers_plus1) reason = "some of the Z stabilizers are not +1"
					else reason = "a X logical operator destroys the logical state"
				}
				if (!data.z_valid) {
					reason = reason ? reason + " and " : ""
					if (!data.if_all_x_stabilizers_plus1) reason += "some of the X stabilizers are not +1"
					else reason += "a Z logical operator destroys the logical state"
				}
				this.correction_fail_reason = reason
			}
			this.$refs.tutorial.on_decoder_run(this.decoder)
			this.refresh()
		},
		onChangeL() {
			this.x_error = this.$refs.qubits.makeSquareArray(this.L)
			this.z_error = this.$refs.qubits.makeSquareArray(this.L)
			this.x_correction = this.$refs.qubits.makeSquareArray(this.L)
			this.z_correction = this.$refs.qubits.makeSquareArray(this.L)
			this.measurement = this.$refs.qubits.makeSquareArray(this.L + 1)
			let that = this
			this.$nextTick(() => { that.refresh() })
		},
		async toggle_tutorial() {
			if (this.tutorial_show) {
				await this.$confirm('Are you sure to quit this tutorial? The current state will be reserved, and you can restart tutorial at any time you want.', 'Message', {
					confirmButtonText: 'Yes',
					cancelButtonText: 'Cancel',
					type: 'error'
				})
			} else {
				await this.$confirm('You are starting interactive tutorial. Your current state may NOT be reserved if continued.', 'Message', {
					confirmButtonText: 'Continue',
					cancelButtonText: 'Cancel',
					type: 'success'
				})
			}
			this.tutorial_show = !this.tutorial_show
		},
		tutorial_last() {
			this.$refs.tutorial.last_interactive()
		},
		tutorial_next() {
			this.$refs.tutorial.next_interactive()
		},
		tutorial_on_change_L(L) {
			this.L = L
			this.code_distance = L
		},
		tutorial_set_errors(errors) {
			this.x_error = errors.x_error
			this.z_error = errors.z_error
			this.refresh()
		},
	},
	watch: {
		L() {
			this.onChangeL()
		},
		display_mode() {
			this.refresh()
		},
		measurement_display() {
			if (!this.measurement_display) {
				for (let i=0; i<=this.L; ++i) for (let j=0; j<=this.L; ++j) this.$refs.qubits.ancillaQubitsErrors[i][j] = 0
			}
			this.refresh()
		},
		decoder() {
			this.$refs.tutorial.on_decoder_changed(this.decoder)
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
