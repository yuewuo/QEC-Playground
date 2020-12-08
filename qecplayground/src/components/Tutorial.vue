<template>
	<el-collapse-transition>
		<div class="holder" v-show="showing">
			<el-steps :active="step" align-center>
				<el-step @click.native="jump_step(0)" title="Quantum Mechanics"></el-step>
				<el-step @click.native="jump_step(1)" title="Quantum Computing"></el-step>
				<el-step @click.native="jump_step(2)" title="Quantum Error Corrrection"></el-step>
				<el-step @click.native="jump_step(3)" title="Surface Code"></el-step>
				<el-step @click.native="jump_step(4)" title="Error Decoder"></el-step>
			</el-steps>
			<el-button type="success" :icon="collapsed ? 'el-icon-arrow-down' : 'el-icon-arrow-up'" circle class="collapse-btn"
				@click="toggle_collapse"></el-button>
			<el-button type="danger" icon="el-icon-close" v-show="running != null" circle class="close-btn"
				@click="close_interactive_part"></el-button>
			<el-collapse-transition>
				<div id="tutorial-has-math" v-show="!collapsed" class="collapse-div" :style="{ 'max-height': max_height + 'px' }">
					<div v-show="step == 0"><!-- Basic Quantum Mechanics -->
						<h1>Introduction</h1>
						<p>Hello! Welcome to the world of quantum computing!</p>
						<p>This is an <strong>interactive tutorial</strong> that can help you get some preliminary knowledge about quantum error correction techniques, more specifically, error correction based on surface code. You can read through all the materials or simply jump to <strong style="color: #2A5CAA;">blue interactive part like below</strong> if you're already expert in this field. At the end of this tutorial you'll be able to manipulate qubit (quantum bit) errors and run error correction algorithms to correct them.</p>
						<el-card shadow="always" :body-style="{ padding: '10px 20px', background: '#2A5CAA' }">
							<p class="interactive-message">Interactive Part: click "Start" button on the right
								<el-button class="interactive-start" type="primary" @click="start_interactive('introduction')"
									:icon="running == 'introduction' ?  'el-icon-loading' : 'none'"
									:disabled="running != null">{{ running == "introduction" ? "Running" : "Start" }}</el-button></p>
						</el-card>
						<p>This is an open-source project at <a href="https://github.com/yuewuo/QEC-Playground" target="_blank">https://github.com/yuewuo/QEC-Playground</a>.</p>
						<h1>Basic Quantum Mechanics</h1>
						<p>In quantum computing, the qubit or quantum bit is the basic unit of quantum information. One can analogize qubit to the classical bit, which is represented by 0 and 1 in classical computing. For a qubit, there are two computational bases denoted as $|0\rangle$ and $|1\rangle$. They are different from classical bit in that the pure state of a qubit $|\psi\rangle$ can be a superposition of the two bases $|\psi\rangle=\alpha|0\rangle+\beta|1\rangle$ where $\alpha,\beta\in\mathbb{C}$. Since qubit pure states $|\psi_0\rangle$ and $|\psi_1\rangle$ are indistinguishable from each other if $|\psi_0\rangle = c|\psi_1\rangle$ given $c\in\mathbb{C}$, one can simply add constrains that $|\alpha|^2+|\beta|^2 = 1$ and $\alpha\in\mathbb{R}$ to make the state unique under each pair of $\alpha, \beta$. This can be reparameterized as    $|\psi\rangle = \cos{\frac{\theta}{2}}|0\rangle + \mathrm{e}^{i\phi}\sin{\frac{\theta}{2}}|1\rangle$ given $\theta\in [0,\pi]$ and $\phi\in [0,2\pi]$. The parameters $\theta$ and $\phi$ can be visualized in spherical coordinates shown below, which is called a <a href="https://en.wikipedia.org/wiki/Bloch_sphere" target="_blank">Bloch sphere</a>.</p>
						<div style="text-align: center;">
							<img src="@/assets/Bloch_sphere.svg"/>
						</div>
						<p>We use this geometrical representation of single qubit in pure state throughout the project. That means, you can view the spheres in GUI as qubits in Bloch sphere.</p>
						<p>Quantum operators can change the state of a qubit, just like classical gates. Analogous to classical NOT gate that maps 0 to 1 and 1 to 0, a Pauli X operator in quantum mechanics maps $|0\rangle$ to $|1\rangle$ and $|1\rangle$ to $|0\rangle$. Thus, Pauli X operator is also known as bit-flip operator that maps any state $\alpha|0\rangle+\beta|1\rangle$ to $\alpha|1\rangle+\beta|0\rangle$. On the contrary, Pauli Z operator does not have correspondence in classical circuit, which is only introduced in the existence of superposition. Pauli Z operator is also known as phase-flip operator because it changes the relative phase of $|1\rangle$ to $\mathrm{e}^{\mathrm{i}\pi}|1\rangle=-|1\rangle$, mapping any state $\alpha|0\rangle+\beta|1\rangle$ to $\alpha|0\rangle-\beta|1\rangle$. With Bloch sphere, one can visualize the effect of Pauli operators as rotating the state along its corresponding axis, for example, Pauli X operator rotates the state along X axis and Pauli Z operator rotates the state along Z axis with $\pi$ angle (180$\deg$). Similarly, Pauli Y operator rotates the state along Y axis, which is equivalent to first applying Z operator then X operator. We use this visualization in our GUI, see the interactive part below.</p>
						<el-card shadow="always" :body-style="{ padding: '10px 20px', background: '#2A5CAA' }">
							<p class="interactive-message">Interactive Part: customize single qubit errors by adding Pauli operators (Pauli errors)
								<el-button class="interactive-start" type="primary" @click="start_interactive('single_qubit')"
									:icon="running == 'single_qubit' ?  'el-icon-loading' : 'none'"
									:disabled="running != null">{{ running == "single_qubit" ? "Running" : "Start" }}</el-button></p>
						</el-card>
						<p>Different from the classical world where measurement is non-destructive to the information, in quantum world measurement usually changes the quantum state by projecting it to a random base of the measurement operator. We can measure the qubit along Z axis, who has the same orthogonal bases $|0\rangle$ and $|1\rangle$ as the computational bases of a qubit. When measuring a qubit along Z axis, it will collapse to one of the orthogonal bases $|0\rangle$ and $|1\rangle$ with probability $|\alpha|^2$ and $|\beta|^2$ respectively. If the measurement result is +1, then after measurement the qubit is in $|0\rangle$ state, otherwise the measurement result is -1 and the qubit is in $|1\rangle$ state after measurement. The measurement of quantum state is destructive in general because it collapses the superposition state $\alpha|0\rangle+\beta|1\rangle$ to either $|0\rangle$ or $|1\rangle$, losing the information of their relative phase $\phi$ and relative amplitude $\tan{\frac{\theta}{2}}=|\frac{\beta}{\alpha}|$.</p>
					</div>
					<div v-show="step == 1"><!-- Quantum Computing -->
						<h1>Quantum Computing</h1>
						<p>The exponentially faster quantum computing comes from the superposition of quantum states. If a quantum computer has $n$ qubits, then there are $2^n$ computational bases $|00..000\rangle$, $|00..001\rangle$, $|00..010\rangle$, ..., $|11..111\rangle$. Unlike a classical circuit with $n$ bits, a quantum circuit with $n$ qubits has $2^n$ bits information because each of its $2^n$ computational bases can be individually superposed. The essence of quantum computing is to use these exponentially larger computational bases to search the result in parallel. Quantum algorithms based on <a href="https://en.wikipedia.org/wiki/Quantum_algorithm#Algorithms_based_on_quantum_walks" target="_blank">quantum walks</a> is proved to give exponential speedups on some tasks by preparing all computational bases and run through a black box to get the result in parallel. Other quantum algorithms based on <a href="https://en.wikipedia.org/wiki/Quantum_algorithm#Algorithms_based_on_the_quantum_Fourier_transform" target="_blank">quantum Fourier transform</a> also perform exponentially faster than classical computers, among which the <a href="https://en.wikipedia.org/wiki/Shor%27s_algorithm" target="_blank">Shor's algorithm</a> solving the integer factorization problem is famous for its threat to today's widely-used encryption technologies.</p>
						<el-card shadow="always" :body-style="{ padding: '10px 20px', background: '#2A5CAA' }">
							<p class="interactive-message">Interactive Part: change the amount of data qubits and see how many computational bases are there
								<el-button class="interactive-start" type="primary" @click="start_interactive('qubit_amount')"
									:icon="running == 'qubit_amount' ?  'el-icon-loading' : 'none'"
									:disabled="running != null">{{ running == "qubit_amount" ? "Running" : "Start" }}</el-button></p>
						</el-card>
					</div>
					<div v-show="step == 2"><!-- Quantum Error Corrrection -->
						<h1>Quantum Error Correction</h1>
						<p>Quantum error correction (QEC) is used in quantum computing to protect quantum information from errors due to decoherence and other quantum noise. Unlike classical computing, there's high probability of error per qubit per time slice (about 1e-3) in the process of quantum computation (e.g. noisy quantum gates, noisy quantum preparation, and even noisy measurements).</p>
						<p>The simplest method of error correction is adding redundancy to qubits, that is to store the qubits information multiple times. When these physical qubit copies disagree in the future, a <a href="https://en.wikipedia.org/wiki/Quantum_error_correction" target="_blank">quantum version repetition code</a> can detect and recover the errors in some circumstances. This is not the same as classical repetition code because direct measurement on each physical qubit will destroy the quantum superposition state. The superposition is the essence of quantum computing supremacy, so a quantum error correction code has to be careful not to destroy the superposition state when doing measurements. Quantum repetition code can only work with single type of Pauli error, so it's not robust to real-world noises composed of all kinds of Pauli errors.</p>
						<p>QEC can detect and further recover from errors under certain conditions, but it also introduces more noisy gates and noisy ancilla qubits (used to assist measurement). That means QEC does not necessarily reduce the error rate given more and more physical qubits if the gates and ancilla qubits to implement error correction introduce even more errors than it could recover from.</p>
						<h1>Fault Tolerant Quantum Computing</h1>
						<p>Fault tolerant quantum computing is a step forward from quantum error correction. It requires that given any small error rate requirement $\epsilon$, we can build a logical qubit with at most $\epsilon$ error rate with bounded amount of physical qubits. Fault tolerant quantum computing is based on QEC but is more strict in terms of the ability to recover from ubiquitous errors. The fault tolerant quantum computing is still a research front and one need to prove a certain QEC code and it's decoder is fault tolerant mathematically or in experiment.</p>
					</div>
					<div v-show="step == 3"><!-- Surface Code -->
						<h1>Topological Code: Surface Code</h1>
						<p>Topological code is a special kind of quantum error correction code that only involves geometrically local gates which means every gate only interacts with adjacent qubits, making it practical to implement. Surface code is one of the topological codes, first proposed as <a href="https://en.wikipedia.org/wiki/Toric_code" target="_blank">toric code</a> in which every qubit is on the surface of a torus. The surface of torus can be mapped onto a 2D square with periodic boundary condition on top, bottom boundary and left, right boundary. Given the current superconducting quantum chip fabricated in 2D, it's hard to implement toric code only with local gates because of the periodic boundary.</p>
						<p>Planar code is a surface code that doesn't require periodic boundary condition, and thus is the most promising one given the current fabrication procedure of superconducting quantum chip. Early version of planar code is a straightforward step from toric code, but it's proven to be sub-optimal in terms of code distance per qubit. Thus, we use the optimized rotated planar code in our project, as shown below.</p>
						<div style="text-align: center;">
							<img style="height: 250px;" src="@/assets/rotated_planar_code_annotated.svg"/>
						</div>
						<p>The rotated planar code is consist of $d\times d$ data qubits. We use only odd number $d$ so that the error correction ability on X and Z Pauli errors is balanced. It has $\frac{(d^2-1)}{2}$ Z stabilizers and $\frac{(d^2-1)}{2}$ X stabilizers, each measures the adjacent 4 or 2 data qubits, according to whether the stabilizer is on the boundary. A demonstration of $d=3$ rotated planar code is shown above. Each piece of rotated planar code forms a single logical qubit, because there are $d^2-1$ stabilizers each locks down a degree of freedom of a data qubit, leaving $d^2-(d^2-1) = 1$ degree of freedom. The goal of quantum error correction is to recover the logical state from random errors, which means the logical state should not be changed after correction. If a logical operator is introduced after correction, then the error correction fails. Logical Pauli X operator and logical Pauli Z operator is shown above as arrows, meaning that if all the qubits on that arrow has Pauli X or Z error then a logical X or Z operator is introduced respectively. Note that the CX gates and H·CX·H gates are used to implement the measurement with the help of stabilizer ancilla qubits, each is a quantum gate operating on two qubits. We'll not dig into the concrete implementation of measurement using those gates but just take the result that the joint measurement of the adjacent 4 or 2 qubits is feasible.</p>
						<div style="text-align: center;">
							<img style="height: 150px;" src="@/assets/Stab.png"/>
						</div>
						<p>Take a Z stabilizer with 4 adjacent data qubits as an example, as show above, it measures $Z_1 Z_2 Z_3 Z_4$ of these data qubits. Suppose at first the measurement result is +1, then if one of the four data qubits has a Pauli X error which bit-flip that data qubit, then the measurement result becomes -1 because $1^3\times(-1)^1 = -1$. However, if two data qubits or all four data qubits have Pauli X errors, then the measurement result is still +1 given that $1^2\times(-1)^2 = 1$ and $(-1)^4 = 1$, meaning that this stabilizer is not able to detect those errors.</p>
						<el-card shadow="always" :body-style="{ padding: '10px 20px', background: '#2A5CAA' }">
							<p class="interactive-message">Interactive Part: add Pauli X errors to data qubits and see the Z stabilizer measurement result
								<el-button class="interactive-start" type="primary" @click="start_interactive('z_measurement')"
									:icon="running == 'z_measurement' ?  'el-icon-loading' : 'none'"
									:disabled="running != null">{{ running == "z_measurement" ? "Running" : "Start" }}</el-button></p>
						</el-card>
						<div style="text-align: center;">
							<img style="height: 300px; margin-top: 30px;" src="@/assets/random_errors.png"/>
						</div>
						<p>The X stabilizer measurements are similar, which only detects odd amounts of Pauli Z errors in the adjacent 4 or 2 data qubits. A demonstration of a random error pattern with both Pauli X errors, Pauli Z errors and Pauli Y errors (having both X and Z errors) is shown above. An error syndrome refers to the stabilizer measurement results. A surface code decoder tries to predict the error pattern from error syndrome and the decoding is successful only if there is no logical operator introduced after the correction and all the stabilizers are back to +1 measurement result after the correction. Note that in practice we only need to remember the errors but do not need to actually correct them by applying quantum gates, but we'll not dig into it here.</p>
						<el-card shadow="always" :body-style="{ padding: '10px 20px', background: '#2A5CAA' }">
							<p class="interactive-message">Interactive Part: play with both types of Pauli errors and see the measurement result
								<el-button class="interactive-start" type="primary" @click="start_interactive('both_errors')"
									:icon="running == 'both_errors' ?  'el-icon-loading' : 'none'"
									:disabled="running != null">{{ running == "both_errors" ? "Running" : "Start" }}</el-button></p>
						</el-card>
					</div>
					<div v-show="step == 4"><!-- Error Decoder -->
						<h1>Surface Code Decoders</h1>
						<p>Existing works have proposed several decoders for surface code, for example Lookup Table (LUT) decoder, Minimum Weight Perfect Matching (MWPM) decoder, Machine Learning (ML) decoder, Tensor Network (TN) decoder and Union-Find (UF) decoder. We use MWPM decoder as a baseline, and propose a new sub-optimal surface code decoder called "naive decoder" to demonstrate how our tool can help developers to visualize failed error corrections and further reason about why this would happen. MWPM decoder is the default decoder in the tutorial to teach people how quantum error correction works interactively.</p>
						<el-card shadow="always" :body-style="{ padding: '10px 20px', background: '#2A5CAA' }">
							<p class="interactive-message">Interactive Part: run surface code decoders
								<el-button class="interactive-start" type="primary" @click="start_interactive('decoders')"
									:icon="running == 'decoders' ?  'el-icon-loading' : 'none'"
									:disabled="running != null">{{ running == "decoders" ? "Running" : "Start" }}</el-button></p>
						</el-card>
					</div>
					<div v-show="step == 5"><!-- All Finished! -->
						<h1>Congratulations!</h1>
						<p>You have finished the tutorials! Have fun with it! &nbsp;
							<el-button size="small" type="danger" @click="finish_tutorials">Quit Tutorials</el-button></p>
						<div style="text-align: center;">
							<img style="width: 300px;" src="@/assets/congratulations.svg"/>
						</div>
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
			running: null,  // running interactive part, should be object if running
			running_idx: 0,
			
			contents: {
				"introduction": [
					{ type: "text", content: "Use the green button on the right-bottom corner of the tutorial page to collapse it." },
					{ type: "text", content: "Great! Now click it again to unfold it. This is useful when you want to switch focusing on the 3D content or the tutorial." },
					{ type: "text", content: "Well done! If you want to quit any interactive tutorial, you can either click the red button on the left-bottom corner of the tutorial, or click 'Quit' button in this card. Now try other interactive tutorials, have fun!" }
				],
				"single_qubit": [
					{ type: "text", content: "Here is a data qubit (orange sphere). Try to rotate the view by press the left mouse and drag. Try to zoom in and out by rolling mouse wheel up and down. Click 'Next' to continue." },
					{ type: "text", content: "Now you can add Pauli errors on this data qubit. To do that, first select 'Toggle X Error (bit-flip error)' in the 'Customize Error Pattern' panel below. Once selected, it will become dark green. Then hover your mouse over the data qubit, so that it jumps up and becomes silver. Click it and you'll see a Pauli X error over it." },
					{ type: "text", content: "You can do the similar thing to add Pauli Z error. Pauli X error is green, Pauli Z error is blue. You can remember this by check the color of the toggle button. Now practice: change the data qubit to a state with both Pauli X error and Pauli Z error. This state is equivalent to adding a Pauli Y error." },
					{ type: "text", content: "Now clear all the errors using 'Clear Error' button in the 'Customize Error Pattern' panel below." },
					{ type: "text", content: "Great job! Now you have learned how to manipulate single qubit errors." },
				],
				"qubit_amount": [
					{ type: "text", content: "With single qubit, there are 2 computational bases $|0\\rangle$ and $|1\\rangle$. Now change the 'Code Distance' to 5 in 'Global Settings' panel by clicking '+' on the right. After that there will be 25 qubits." },
					{ type: "text", content: "How many computational bases are there with 25 qubits? Click 'Next' to check the answer." },
					{ type: "text", content: "The answer is $2^{25}$. Cool! You've finished half of the interactive tutorials." },
				],
				"z_measurement": [
					{ type: "text", content: "The blue sphere in the middle is an ancilla qubit that assists Z stabilizer measurement. If the measurement result of $Z_1 Z_2 Z_3 Z_4$ changes from +1 to -1, it becomes red indicating there is some Pauli X errors on the adjacent 4 data qubits. Try add a Pauli X error to any of the data qubits." },
					{ type: "text", content: "You can see that the stabilizer becomes red because there is one Pauli X error nearby. Now try to add one more Pauli X errors to any of the data qubits. It should have 2 Pauli X errors in total after doing that." },
					{ type: "text", content: "It becomes blue again, indicating that the stabilizer is no longer be able to detect these errors! But don't worry, we can later see that large surface code can suppress the probability of undetectable errors. Now try to add Pauli X errors to all 4 data qubits, and try to find the pattern of measurement result." },
					{ type: "text", content: "Perfect! You've learned what is the measurement result of stabilizers. Just one thing to remember, Z stabilizers only detect odd number of adjacent Pauli X errors, and X stabilizers only detect odd number of adjacent Pauli Z errors. You can also remember them by color, that is, blue sphere only detects green errors and green sphere only detects blue errors." },
				],
				"both_errors": [
					{ type: "text", content: "You can now try to play with both types of Pauli errors and their measurement results. Try to add more than one errors on data qubit and still keep all the stabilizers at +1 result. See what is the minimum amount of errors to do that. Tips: how about adding logical operators? Is it minimum?" },
					{ type: "text", content: "Wonderful! You've got the core of surface code. With larger and larger code distance, it's exponentially unlikely that an undetectable logical operator is introduced with random distributed errors, thus the logical state is protected." },
				],
				"decoders": [
					{ type: "text", content: "Try to add some random errors. After doing that, click 'Run Correction' in 'Run Error Correction' panel. It will request remote server to decode the error syndrome and return the error correction pattern. Please have a look at the 'Display' option in 'Global Settings' panel, it will change automatically after you click the 'Run Correction' button." },
					{ type: "text", content: "Now you can see the display mode is set to 'Corrected', which means you're viewing the combination of error pattern and correction pattern. In this display mode, all the stabilizer should return to +1 (no red stabilizers) and you will see a message above the 'Run Correction' button you just clicked. It will analyze whether the error correction is successful or not, and give reasons of failure. Now select 'Naive Decoder' as decoder." },
					{ type: "text", content: "We've found an error pattern (already updated to your data qubits) that fails with 'Naive Decoder' but succeeds with 'MWPM Decoder'. Try to run the two decoders one by one and see their difference. Click 'Next' to continue." },
					{ type: "text", content: "You can easily fool the decoder by directly adding logical errors to the data qubits. Since the decoder will simply assume that there is no error at all, an logical error is definitely introduced after correction. Click 'Next' to continue." },
					{ type: "text", content: "Note that although we're applying the error correction to data qubits to help you visualize the result, in practice we don't need to do that. We just need to remember the error happened before and change the behavior of quantum gates, measurements, etc. Well, that's the end of interactive tutorial. Have fun with it!" },
				],
			},
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
			this.step = 0
		},
		stop_tutorial() {
			// remove all the strange settings of GUI
		},
		toggle_collapse() {
			if (this.running == "introduction" && this.running_idx == 0 && this.collapsed == false) {
				this.next_interactive()
			}
			if (this.running == "introduction" && this.running_idx == 1 && this.collapsed == true) {
				this.next_interactive()
			}
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
		jump_step(step) {
			this.step = step
		},
		finish_tutorials() {
			this.$emit('showing', false)
		},
		close_interactive_part() {
			this.running = null
			this.collapsed = false  // show up the tutorial so that user can continue reading it
		},
		start_interactive(name, idx = 0) {
			this.running = name
			this.running_idx = idx
			let hideZancilla = false
			let hideXancilla = false
			if (name == "single_qubit" && idx == 0) {
				this.$emit("L", 1)  // set to single qubit
				this.collapsed = true
			}
			if (name == "qubit_amount" && idx == 0) {
				this.$emit("L", 1)  // set to single qubit
				this.collapsed = true
				hideZancilla = true
				hideXancilla = true
			}
			if (name == "z_measurement" && idx == 0) {
				this.$emit("L", 2)
				this.collapsed = true
				hideXancilla = true
			}
			if (name == "both_errors" && idx == 0) {
				this.$emit("L", 5)
				this.collapsed = true
			}
			if (name == "decoders" && idx == 0) {
				this.$emit("L", 5)
				this.collapsed = true
			}
			this.$emit("hideZancilla", hideZancilla)
			this.$emit("hideXancilla", hideXancilla)
			this.update_interactive()
		},
		async vue_next_tick() {
			let that = this
			await new Promise((resolve, reject) => {
				that.$nextTick(() => { resolve() })
			})
		},
		makeSquareArray(width, valueGen=(()=>0)) {
			const array = []
			for (let i=0; i<width; ++i) {
				const row = []
				for (let j=0; j<width; ++j) {
					row.push(valueGen(i,j))
				}
				array.push(row)
			}
			return array
		},
		async update_interactive() {
			if (this.running == null) return
			let name = this.running
			let idx = this.running_idx
			if (name == "decoders" && idx == 2) {
				this.$emit("L", 5)
				await this.vue_next_tick()
				let x_error = this.makeSquareArray(5)
				let z_error = this.makeSquareArray(5)
				x_error[1][1] = 1
				x_error[1][2] = 1
				x_error[2][3] = 1
				this.$emit("set_errors", {
					x_error, z_error
				})
			}
		},
		last_interactive() {
			if (this.running == null) return
			let idx = this.running_idx - 1
			if (idx < 0) idx = 0
			this.running_idx = idx
			this.update_interactive()
		},
		next_interactive() {
			if (this.running == null) return
			const name = this.running
			let idx = this.running_idx + 1
			if (idx >= this.contents[this.running].length) {
				this.close_interactive_part()
				return
			}
			this.running_idx = idx
			this.update_interactive()
		},
		on_data_qubit_changed(x_error, z_error, measurement) {
			if (this.running == "single_qubit") {
				if (this.running_idx == 1 && x_error[0][0] == 1) {
					this.next_interactive()
				}
				if (this.running_idx == 2 && z_error[0][0] == 1 && x_error[0][0] == 1) {
					this.next_interactive()
				}
				if (this.running_idx == 3 && z_error[0][0] == 0 && x_error[0][0] == 0) {
					this.next_interactive()
				}
			}
			if (this.running == "qubit_amount" && this.running_idx == 0 && x_error.length == 5) {
				this.next_interactive()
			}
			if (this.running == "z_measurement") {
				let cnt = 0
				for (let i=0; i < x_error.length; ++i) {
					for (let j=0; j < x_error[i].length; ++j) {
						cnt += x_error[i][j]
					}
				}
				if (this.running_idx == 0 && cnt == 1) {
					this.next_interactive()
				}
				if (this.running_idx == 1 && cnt == 2) {
					this.next_interactive()
				}
				if (this.running_idx == 2 && cnt == 4) {
					this.next_interactive()
				}
			}
			if (this.running == "both_errors") {
				let error_cnt = 0
				for (let i=0; i < x_error.length; ++i) {
					for (let j=0; j < x_error[i].length; ++j) {
						error_cnt += x_error[i][j]
						error_cnt += z_error[i][j]
					}
				}
				let measurement_error_cnt = 0
				for (let i=0; i < measurement.length; ++i) {
					for (let j=0; j < measurement[i].length; ++j) {
						measurement_error_cnt += measurement[i][j]
					}
				}
				if (error_cnt != 0 && measurement_error_cnt == 0) {
					this.next_interactive()
				}
			}
		},
		on_decoder_run(decoder_name) {
			if (this.running == "decoders") {
				if (this.running_idx == 0) {
					this.next_interactive()
				}
			}
		},
		on_decoder_changed(decoder_name) {
			if (this.running == "decoders") {
				if (this.running_idx == 1 && decoder_name == "naive_decoder") {
					this.next_interactive()
				}
			}
		}
	},
	watch: {
		show() {
			this.showing = this.show
			if (this.showing) {
				this.start_tutorial()
			} else {
				this.stop_tutorial()
			}
		},
		running() {
			this.$emit("running", this.running)
		},
		running_idx() {
			this.$emit("running_idx", this.running_idx)
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

.close-btn {
	position: absolute;
	bottom: 5px;
	left: -5px;
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

.interactive-message {
	margin: 10px 120px 10px 0;
	color: white;
	position: relative;
}

.interactive-start {
	position: absolute;
	right: -120px;
	top: -5px;
}

</style>
