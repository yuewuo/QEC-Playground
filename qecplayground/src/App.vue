<template>
	<div id="app">
		<MainQubits class="main-qubits" :panelWidth="480" :enableStats="enableStats" :decoderServerRootUrl="decoderServerRootUrl"/>
		<div class="control-panel">
			<h1 class="title"><img src="@/assets/logo.png" class="logo"/>QEC Playground</h1>
			<p>This is an educational tool for Quantum Error Correction (QEC). You can learn the currently most promising QEC scheme called surface code (planar code) by following the introduction tutorial and then trying different error patterns interactively.</p>
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
		
	},
}
</script>

<style>

#app {
	font-family: 'Avenir', Helvetica, Arial, sans-serif;
	-webkit-font-smoothing: antialiased;
	-moz-osx-font-smoothing: grayscale;
	text-align: center;
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

</style>
