<template>
  <div class="main" id="main_qubits_container">
	
  </div>
</template>

<script>
import * as THREE from 'three'
import Stats from 'three/examples/jsm/libs/stats.module.js'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'

export default {
	name: 'MainQubits',
	props: {
		L: {
			type: Number,
			default: 11,
		},

		enableStats: {  // performance monitor
			type: Boolean,
			default: true
		},
		pointSize: {  // wave-like point cloud. point size
			type: Number,
			default: 0.011
		},
		pointDense: {  // wave-like point cloud. how many points in a cycle
			type: Number,
			default: 60,
		},
		panelWidth: {
			type: Number,
			default: 480
		},
		qubitInterval: {
			type: Number,
			default: 1.0
		},
		waveHeight: {
			type: Number,
			default: 1.0
		},
		dataQubitWaveColor: {
			type: Object,
			default: () => new THREE.Color( 1, 0.65, 0 )
		},
		dataQubitWaveConcetrate: {
			type: Number,
			default: 1.3
		},
		xStabWaveColor: {
			type: Object,
			default: () => new THREE.Color( 0, 0.75, 1 )
		},
		xStabWaveConcetrate: {
			type: Number,
			default: 4
		},
		zStabWaveColor: {
			type: Object,
			default: () => new THREE.Color( 0, 0.8, 0 )
		},
		zStabWaveConcetrate: {
			type: Number,
			default: 4
		},
	},
	data() {
		return {
			qubitsGroup: undefined,
		}
	},
	mounted() {
		const scene = new THREE.Scene()

		// add camera and renderer
		const camera = new THREE.PerspectiveCamera( 45, (window.innerWidth-this.panelWidth) / window.innerHeight, 0.1, 10000 )
		const initCameraRatio = this.L * 0.6
		camera.position.set( 1 * initCameraRatio, 1 * initCameraRatio, 2 * initCameraRatio )
		camera.lookAt( scene.position )
		camera.updateMatrix()
		const renderer = new THREE.WebGLRenderer({ antialias: true })
		renderer.setPixelRatio( window.devicePixelRatio )
		renderer.setSize( window.innerWidth-this.panelWidth, window.innerHeight )
		window.addEventListener( 'resize', () => {
			camera.aspect = (window.innerWidth-this.panelWidth) / window.innerHeight
			camera.updateProjectionMatrix()
			renderer.setSize( window.innerWidth-this.panelWidth, window.innerHeight )
		}, false )
		let container = document.getElementById('main_qubits_container')
		let orbitControl = new OrbitControls( camera, renderer.domElement )
		container.appendChild(renderer.domElement)


		// add qubits
		this.generatePointCloud()
	
		const pointCloud = this.generatePointCloud(100)
		scene.add( pointCloud )
		console.log(pointCloud)

		// const sphereGeometry = new THREE.SphereBufferGeometry( 5, 32, 32 )
		// const sphereMaterial = new THREE.MeshBasicMaterial( { color: 0xff0000 } )
		// const sphere = new THREE.Mesh( sphereGeometry, sphereMaterial )
		// scene.add( sphere )

		
		// start rendering
		let stats = undefined
		const animate = () => {
			requestAnimationFrame( animate )
			if (stats) stats.update()  // update stats if exists
			renderer.render( scene, camera )
		}
		animate()

		// add stats if enabled
		if (this.enableStats) {
			stats = new Stats()
			container.appendChild( stats.dom )
		}
	},
	methods: {
		generatePointCloud() {
			const geometry = new THREE.BufferGeometry()
			const width = this.L * this.pointDense
			const numPoints = width * width
			const positions = new Float32Array( numPoints * 3 )
			const colors = new Float32Array( numPoints * 3 )
			let k = 0
			const bias = -this.qubitInterval * this.L / 2
			for ( let i = 0; i < width; i ++ ) {
				for ( let j = 0; j < width; j ++ ) {
					const x = bias + i * this.qubitInterval / this.pointDense
					const y = - ( Math.cos( i * Math.PI * 2 / this.pointDense ) + Math.cos( j * Math.PI * 2 / this.pointDense ) ) * (this.waveHeight / 4)
					const z = bias + j * this.qubitInterval / this.pointDense
					positions[ 3 * k ] = x
					positions[ 3 * k + 1 ] = y
					positions[ 3 * k + 2 ] = z
					let dataQubitIntensity = ( 2 - Math.cos( (i+j) / this.pointDense * Math.PI ) - Math.cos( (i-j) / this.pointDense * Math.PI + Math.PI ) ) / 4
					dataQubitIntensity = Math.max(0, 1 - (1-dataQubitIntensity) * this.dataQubitWaveConcetrate)
					let xStabIntensity = ( 2 - Math.cos( (i / this.pointDense - 0.5) * Math.PI ) - Math.cos( (j / this.pointDense + 0.5) * Math.PI ) ) / 4
					xStabIntensity = Math.max(0, 1 - (1-xStabIntensity) * this.xStabWaveConcetrate)
					let zStabIntensity = ( 2 - Math.cos( (i / this.pointDense + 0.5) * Math.PI ) - Math.cos( (j / this.pointDense - 0.5) * Math.PI ) ) / 4
					zStabIntensity = Math.max(0, 1 - (1-zStabIntensity) * this.zStabWaveConcetrate)
					colors[ 3 * k ] = this.dataQubitWaveColor.r * dataQubitIntensity + this.xStabWaveColor.r * xStabIntensity
						+ this.zStabWaveColor.r * zStabIntensity
					colors[ 3 * k + 1 ] = this.dataQubitWaveColor.g * dataQubitIntensity + this.xStabWaveColor.g * xStabIntensity
						+ this.zStabWaveColor.g * zStabIntensity
					colors[ 3 * k + 2 ] = this.dataQubitWaveColor.b * dataQubitIntensity + this.xStabWaveColor.b * xStabIntensity
						+ this.zStabWaveColor.b * zStabIntensity
					k ++
				}
			}
			geometry.setAttribute( 'position', new THREE.BufferAttribute( positions, 3 ) )
			geometry.setAttribute( 'color', new THREE.BufferAttribute( colors, 3 ) )
			geometry.computeBoundingSphere()
			const material = new THREE.PointsMaterial( { size: this.pointSize, vertexColors: true } )
			return new THREE.Points( geometry, material )
		},
	},
}
</script>

<style scoped>

</style>
