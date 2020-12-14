<template>
	<div id="app">
		<FaultTolerantView class="main-qubits" :panelWidth="480" :L="L" :T="T"></FaultTolerantView>
		<div class="control-panel no-scrollbar">
			<div style="text-align: center;">
				<h1 class="title"><img src="@/assets/logo.png" class="logo"/>QEC Playground</h1>
				<p>This is an educational tool for Quantum Error Correction (QEC). You can learn the currently most promising QEC scheme called surface code (planar code) by following the introduction tutorial and then trying different error patterns interactively.</p>
			</div>
			<div>
				<p>something</p>
			</div>
			
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
		onChangeL() {

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
