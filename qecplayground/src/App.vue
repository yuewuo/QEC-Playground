<template>
	<div id="app">
		<MainQubits class="main-qubits" msg="Welcome to Your Vue.js App" :panelWidth="0"/>
	</div>
</template>

<script>
import MainQubits from './components/MainQubits.vue'

export default {
	name: 'app',
	components: {
		MainQubits,
	},
	data() {
		return {
			
		}
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

</style>
