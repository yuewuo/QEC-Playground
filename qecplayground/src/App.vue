<template>
	<div id="app">
		<MainQubits class="main-qubits" :panelWidth="480" :enableStats="enableStats" :decoderServerRootUrl="decoderServerRootUrl"
			:toggleXError="toggle_X_error" :toggleZError="toggle_Z_error"/>
		<div class="control-panel">
			<div style="text-align: center;">
				<h1 class="title"><img src="@/assets/logo.png" class="logo"/>QEC Playground</h1>
				<p>This is an educational tool for Quantum Error Correction (QEC). You can learn the currently most promising QEC scheme called surface code (planar code) by following the introduction tutorial and then trying different error patterns interactively.</p>
			</div>
			<el-card>
				<div slot="header" class="clearfix">
					<span>Customize Error Pattern</span>
					<el-button style="float: right; padding: 3px 0" type="text">help</el-button>
				</div>
				<div>
					<el-button type="success" class="full-width-button" :plain="!toggle_X_error" @click="enable_toggle_error(true)">
						Toggle X Error (bit-flip error)</el-button>
					<br><br>
					<el-button type="primary" class="full-width-button" :plain="!toggle_Z_error" @click="enable_toggle_error(false)">
						Toggle Z Error (phase-flip error)</el-button>
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

.full-width-button {
	width: 100%;
}

</style>
