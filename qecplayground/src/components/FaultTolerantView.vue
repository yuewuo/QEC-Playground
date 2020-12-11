<template>
  <div class="main" id="fault_tolerant_view_container"></div>
</template>

<script>
import * as THREE from 'three'
import Stats from 'three/examples/jsm/libs/stats.module.js'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'

export default {
	name: 'FaultTolerantView',
	props: {
		L: {
			type: Number,
			default: 5,
		},
		H: {
			type: Number,
			default: 5,
		},
		
		
		panelWidth: {
			type: Number,
			default: 480
		},
		dataQubitColor: {
			type: Object,
			default: () => new THREE.Color( 1, 0.65, 0 )
        },
        enableStats: {
            type: Boolean,
            default: true,
        },
        enableBackground: {
            type: Boolean,
            default: true,
        },
	},
	data() {
		return {
			three: { },  // save necessary THREE.js objects
			internals: { },  // internal data for control

			// controllable parameters for visualization
			dataQubitsDynamicYBias: [ ],  // [L][L] float numbers
			zDataQubitsErrors: [ ],  // [L][L] 0 ~ 1
			xDataQubitsErrors: [ ],  // [L][L] 0 ~ 1
			ancillaQubitsErrors: [ ],  // [L+1][L+1] 0 ~ 1
		}
	},
	mounted() {
		window.$ftview = this  // for fast debugging

		const scene = new THREE.Scene()
		this.three.scene = scene
		this.three.clock = new THREE.Clock()
		this.three.clockAbs = new THREE.Clock()

		// add camera and renderer
		const windowWidth = window.innerWidth - this.panelWidth
        const windowHeight = window.innerHeight
        console.log(windowWidth)
        console.log(windowHeight)
		const camera = new THREE.PerspectiveCamera( 75, windowWidth / window.innerHeight, 0.1, 10000 )
		this.three.camera = camera
		const initCameraRatio = this.L * 0.5
		camera.position.set( -2 * initCameraRatio, 1 * initCameraRatio, 1 * initCameraRatio )
		camera.lookAt( scene.position )
		camera.updateMatrix()
		const renderer = new THREE.WebGLRenderer({ antialias: true })
		this.three.renderer = renderer
		renderer.setPixelRatio( window.devicePixelRatio )
		renderer.setSize( windowWidth, windowHeight )
		let container = document.getElementById('fault_tolerant_view_container')
		let orbitControl = new OrbitControls( camera, renderer.domElement )
		container.appendChild(renderer.domElement)

		// support for resize
		let that = this
		window.addEventListener( 'resize', () => {
			const windowWidth = window.innerWidth - this.panelWidth
			const windowHeight = window.innerHeight
			that.three.camera.aspect = windowWidth / windowHeight
			that.three.camera.updateProjectionMatrix()
			renderer.setSize( windowWidth, windowHeight )
		}, false )

		// add background
		if (this.enableBackground) {
			const loader = new THREE.CubeTextureLoader();
			const texture = loader.load([
				'/px.jpg', '/nx.jpg', '/py.jpg', '/ny.jpg', '/pz.jpg', '/nz.jpg',
			])
			scene.background = texture
		} else {
            scene.background = new THREE.Color(0xFFFFFF)
        }

		// add stats if enabled
		if (this.enableStats) {
			this.three.stats = new Stats()
			container.appendChild( this.three.stats.dom )
        }
        
        this.build_standard_planar_code()

		// start rendering
		this.animate()

	},
	methods: {
		animate() {
			requestAnimationFrame( this.animate )  // call first
			const delta = this.three.clock.getDelta()
			const absTime = this.three.clockAbs.getElapsedTime()
			if (this.three.stats) this.three.stats.update()  // update stats if exists
			this.three.renderer.render( this.three.scene, this.three.camera )
        },
        build_standard_planar_code() {
            for (let h=0; h<this.H; ++h) {
                for (let i=0; i<this.L; ++i) {
                    for (let j=0; j<this.L; ++j) {
                        
                    }
                }
            }
        },
        build_rotated_planar_code() {
            // TODO
        },
	},
	watch: {
        
	},
}
</script>

<style scoped>

.main {
	background: red;
}

</style>
