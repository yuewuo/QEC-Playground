<template>
	<el-collapse-transition>
		<div class="holder" v-show="showing">
			<el-steps :active="step" align-center>
				<el-step title="Quantum Computing"></el-step>
				<el-step title="Qubit Operation"></el-step>
				<el-step title="Stabilizer Measurement"></el-step>
				<el-step title="Surface Code"></el-step>
				<el-step title="Error Correction"></el-step>
			</el-steps>
			<el-button type="success" :icon="collapsed ? 'el-icon-arrow-down' : 'el-icon-arrow-up'" circle class="collapse-btn"
				@click="toggle_collapse"></el-button>
			<el-collapse-transition>
				<div id="tutorial-has-math" v-show="!collapsed" class="collapse-div" :style="{ 'max-height': max_height + 'px' }">
					<div v-show="step == 0"><!-- Quantum Computing -->
						<!-- template <vue-mathjax formula=""></vue-mathjax> -->
						<h1>Introduction</h1>
						<p>Hello! Welcome to the world of quantum computing. TODO: add contents to let readers jump to interactive parts without reading those long paragraphs</p>
						<h1>Basic Quantum Mechanics</h1>
						<p>In quantum computing, the qubit or quantum bit is the basic unit of quantum information. One can analogize qubit to the classical bit, which is represented by 0 and 1 in classical computing. For a qubit, there are two computational bases denoted as $|0\rangle$ and $|1\rangle$. They are different from classical bit in that the pure state of a qubit $|\psi\rangle$ can be a superposition of the two bases $|\psi\rangle=\alpha|0\rangle+\beta|1\rangle$ where $\alpha,\beta\in\mathbb{C}$. Since qubit pure states $|\psi_0\rangle$ and $|\psi_1\rangle$ are indistinguishable from each other if $|\psi_0\rangle = c|\psi_1\rangle$ given $c\in\mathbb{C}$, one can simply add constrains that $|\alpha|^2+|\beta|^2 = 1$ and $\alpha\in\mathbb{R}$ to make the state unique under each pair of $\alpha, \beta$. This can be reparameterized as    $|\psi\rangle = \cos{\frac{\theta}{2}}|0\rangle + \mathrm{e}^{i\phi}\sin{\frac{\theta}{2}}|1\rangle$ given $\theta\in [0,\pi]$ and $\phi\in [0,2\pi]$. The parameters $\theta$ and $\phi$ can be visualized in spherical coordinates shown below, which is called a <a href="https://en.wikipedia.org/wiki/Bloch_sphere" target="_blank">Bloch sphere</a>.</p>
						<div style="text-align: center;">
							<img src="@/assets/Bloch_sphere.svg"/>
						</div>
						<p>We use this geometrical representation of single qubit in pure state throughout the project. That means, you can view the spheres in GUI as qubits in Bloch sphere.</p>
						<p>Quantum operators can change the state of a qubit, just like classical gates. Analogous to classical NOT gate that maps 0 to 1 and 1 to 0, a Pauli X operator in quantum mechanics maps $|0\rangle$ to $|1\rangle$ and $|1\rangle$ to $|0\rangle$. Thus, Pauli X operator is also known as bit-flip operator that maps any state $\alpha|0\rangle+\beta|1\rangle$ to $\alpha|1\rangle+\beta|0\rangle$. On the contrary, Pauli Z operator does not have correspondence in classical circuit, which is only introduced in the existence of superposition. Pauli Z operator is also known as phase-flip operator because it changes the relative phase of $|1\rangle$ to $\mathrm{e}^{\mathrm{i}\pi}|1\rangle=-|1\rangle$, mapping any state $\alpha|0\rangle+\beta|1\rangle$ to $\alpha|0\rangle-\beta|1\rangle$. With Bloch sphere, one can visualize the effect of Pauli operators as rotating the state along its corresponding axis, for example, Pauli X operator rotates the state along X axis and Pauli Z operator rotates the state along Z axis with $\pi$ angle (180$\deg$). Similarly, Pauli Y operator rotates the state along Y axis, which is equivalent to first applying Z operator then X operator. We use this visualization in our GUI, see the interactive part below.</p>
						
					</div>
					<div v-show="step == 1"><!-- Qubit Operation -->

					</div>
					<div v-show="step == 2"><!-- Stabilizer Measurement -->

					</div>
					<div v-show="step == 3"><!-- Surface Code -->

					</div>
					<div v-show="step == 4"><!-- Error Correction -->

					</div>
					<div v-show="step == 5"><!-- All Finished! -->

					</div>
					<el-button icon="el-icon-arrow-left" class="left-btn" plain @click="step_back" :disabled="step==0"></el-button>
					<el-button icon="el-icon-arrow-right" class="right-btn" plain @click="step_forward" :disabled="step==5"></el-button>
				</div>
			</el-collapse-transition>
		</div>
	</el-collapse-transition>
</template>

<script>
export default {
	name: 'Tutorial',
	props: {
		show: {
			type: Boolean,
			default: false,
		},
	},
	data() {
		return {
			step: 0,
			showing: false,
			collapsed: false,

			max_height: 300,  // this will be changed when window resize
		}
	},
	mounted() {
		window.$tutorial = this  // for fast debugging
		this.showing = this.show
		this.update_size()
		window.addEventListener( 'resize', this.update_size, false )
		this.MathjaxConfig.MathQueue("tutorial-has-math")
	},
	methods: {
		start_tutorial() {  // reinitialize
			this.collapsed = false
		},
		toggle_collapse() {
			this.collapsed = !this.collapsed
		},
		update_size() {
			const windowHeight = window.innerHeight
			this.max_height = windowHeight - 140
		},
		step_back() {
			this.step -= 1
			if (this.step < 0) this.step = 0
		},
		step_forward() {
			this.step += 1
			if (this.step > 5) this.step = 5
		},
	},
	watch: {
		show() {
			this.showing = this.show
			if (this.showing) {
				this.start_tutorial()
			}
		},
	},
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style lang="less" scoped>

.holder {
	position: fixed;
	z-index: 1000;
	background: white;
	top: 20px;
	left: 20px;
	right: calc(20px + 480px);
	padding: 20px;
	border-radius: 20px;
	box-shadow: 0 2px 4px rgba(0, 0, 0, .12), 0 0 6px rgba(0, 0, 0, .04);
	overflow: auto;
}

.collapse-btn {
	position: absolute;
	bottom: 5px;
	right: 5px;
}

.collapse-div {
	// background: red;
	overflow: auto;
	margin-left: 50px;
	margin-right: 50px;
	min-height: 150px;
}

.left-right-btn {
	z-index: -1;
	position: absolute;
	height: calc(100% - 105px);
	width: 60px;
	top: 80px;
}

.left-btn {
	.left-right-btn();
	left: 0px;
}

.right-btn {
	.left-right-btn();
	right: 0px;
}

p {
	line-height: 150%;
	font-size: 110%;
}

</style>
