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
				<div v-show="!collapsed" class="collapse-div" :style="{ 'max-height': max_height + 'px' }">
					<div v-if="step == 0"><!-- Quantum Computing -->
						<p v-for="count in 100">count: {{ count }}</p>
					</div>
					<div v-if="step == 1"><!-- Qubit Operation -->

					</div>
					<div v-if="step == 2"><!-- Stabilizer Measurement -->

					</div>
					<div v-if="step == 3"><!-- Surface Code -->

					</div>
					<div v-if="step == 4"><!-- Error Correction -->

					</div>
					<div v-if="step == 5"><!-- All Finished! -->

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
	},
	methods: {
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

</style>
