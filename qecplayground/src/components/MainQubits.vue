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
			default: 5,
		},

		usePerspectiveCamera: {
			type: Boolean,
			default: true
		},
		dataQubitColor: {
			type: Number,
			default: 0xFFA600
		},
		zStabQubitColor: {
			type: Number,
			default: 0x00BFFF
		},
		xStabQubitColor: {
			type: Number,
			default: 0x00CC00
		},
		dataQubitYBias: {  // Y bias of data qubits
			type: Number,
			default: 0.3
		},
		dataQubitSize: {  // radius of data qubits
			type: Number,
			default: 0.2
		},
		enableBackground: {  // fancy background
			type: Boolean,
			default: true
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
		zStabWaveColor: {
			type: Object,
			default: () => new THREE.Color( 0, 0.75, 1 )
		},
		zStabWaveConcetrate: {
			type: Number,
			default: 4
		},
		xStabWaveColor: {
			type: Object,
			default: () => new THREE.Color( 0, 0.8, 0 )
		},
		xStabWaveConcetrate: {
			type: Number,
			default: 4
		},
	},
	data() {
		return {
			three: { },  // save necessary THREE.js objects
			internals: { },  // internal data for control
		}
	},
	mounted() {
		const scene = new THREE.Scene()
		this.three.scene = scene

		// add camera and renderer
		const windowWidth = window.innerWidth-this.panelWidth
		const windowHeight = window.innerHeight
		const camera = this.usePerspectiveCamera ? 
			new THREE.PerspectiveCamera( 75, windowWidth / window.innerHeight, 0.1, 10000 ) :
			new THREE.OrthographicCamera( windowWidth / windowHeight * -3, windowWidth / windowHeight * 3, 3, -3, 0.1, 10000 )
		this.three.camera = camera
		const initCameraRatio = this.L * 0.5
		camera.position.set( -2 * initCameraRatio, 1 * initCameraRatio, 1 * initCameraRatio )
		camera.lookAt( scene.position )
		camera.updateMatrix()
		const renderer = new THREE.WebGLRenderer({ antialias: true })
		this.three.renderer = renderer
		renderer.setPixelRatio( window.devicePixelRatio )
		renderer.setSize( windowWidth, windowHeight )
		window.addEventListener( 'resize', () => {
			camera.aspect = windowWidth / windowHeight
			camera.updateProjectionMatrix()
			renderer.setSize( windowWidth, windowHeight )
		}, false )
		let container = document.getElementById('main_qubits_container')
		let orbitControl = new OrbitControls( camera, renderer.domElement )
		container.appendChild(renderer.domElement)

		// add background
		if (this.enableBackground) {
			const loader = new THREE.CubeTextureLoader();
			const texture = loader.load([
				'/px.jpg', '/nx.jpg', '/py.jpg', '/ny.jpg', '/pz.jpg', '/nz.jpg',
			]);
			scene.background = texture;
		}

		// add raycaster
		const raycaster = new THREE.Raycaster()
		this.three.raycaster = raycaster
		const mouse = new THREE.Vector2( 1, 1 )
		this.three.mouse = mouse
		document.addEventListener( 'mousemove', event => {
			event.preventDefault()
			mouse.x = ( event.clientX / windowWidth ) * 2 - 1
			mouse.y = - ( event.clientY / windowHeight ) * 2 + 1
		}, false )

		// add qubits
		const pointCloud = this.generatePointCloud()
		scene.add( pointCloud )
		this.generateDataQubits()
		this.generateAncillaQubits()
		const clickableGroup = new THREE.Group()
		for (const obj of this.internals.dataQubitsArray) clickableGroup.add(obj)
		for (const obj of this.internals.ancillaQubitsArray) clickableGroup.add(obj)
		scene.add( clickableGroup )
		this.internals.clickableGroup = clickableGroup

		// add stats if enabled
		if (this.enableStats) {
			this.three.stats = new Stats()
			container.appendChild( this.three.stats.dom )
		}

		// start rendering
		this.animate()

	},
	methods: {
		animate() {
			requestAnimationFrame( this.animate )  // call first
			this.three.raycaster.setFromCamera( this.three.mouse, this.three.camera )
			const intersection = this.three.raycaster.intersectObject( this.internals.clickableGroup, true )
			if ( intersection.length > 0 ) {
				const object = intersection[0].object
				console.log(object)
			}
			if (this.three.stats) this.three.stats.update()  // update stats if exists
			this.three.renderer.render( this.three.scene, this.three.camera )
		},
		generatePointCloud() {
			const geometry = new THREE.BufferGeometry()
			const extend = Math.round(this.pointDense * 0.7)
			const width = (this.L - 1) * this.pointDense + 2 * extend
			const numPoints = width * width
			const positions = new Float32Array( numPoints * 3 )
			const colors = new Float32Array( numPoints * 3 )
			let k = 0
			const bias = (this.L - 1) / 2
			const start = - extend
			const end = width - extend
			for ( let i = start; i < end; i++ ) {
				for ( let j = start; j < end; j++ ) {
					let [x, y, z, r, g, b] = this.wavePositionColor(i / this.pointDense, j / this.pointDense)
					x -= bias
					z -= bias
					y = (y - 1) * this.waveHeight
					positions[ k ] = x
					positions[ k + 1 ] = y
					positions[ k + 2 ] = z
					colors[ k ] = r
					colors[ k + 1 ] = g
					colors[ k + 2 ] = b
					k += 3
				}
			}
			geometry.setAttribute( 'position', new THREE.BufferAttribute( positions, 3 ) )
			geometry.setAttribute( 'color', new THREE.BufferAttribute( colors, 3 ) )
			geometry.computeBoundingSphere()
			const material = new THREE.PointsMaterial( { size: this.pointSize, vertexColors: true } )
			return new THREE.Points( geometry, material )
		},
		wavePositionColor(i, j) {
			const y = ( 2 + Math.cos( i * Math.PI * 2 ) + Math.cos( j * Math.PI * 2 ) ) / 4
			const z = i
			const x = j
			let dataQubitIntensity = ( 2 + Math.cos( i * Math.PI * 2 ) + Math.cos( j * Math.PI * 2 ) ) / 4
			dataQubitIntensity = Math.max(0, 1 - (1-dataQubitIntensity) * this.dataQubitWaveConcetrate)
			if (i < -0.5 || i > this.L - 0.5 || j < -0.5 || j > this.L - 0.5) {
				dataQubitIntensity = 0
			}
			let zStabIntensity = 0
			if (j >= 0 && j <= this.L - 1) {
				zStabIntensity = ( 2 + Math.cos( (i + j - 1) * Math.PI ) + Math.cos( (i - j) * Math.PI ) ) / 4
				zStabIntensity = Math.max(0, 1 - (1-zStabIntensity) * this.zStabWaveConcetrate)
			}
			let xStabIntensity = 0
			if (i >= 0 && i <= this.L - 1) {
				xStabIntensity = ( 2 + Math.cos( (i + j) * Math.PI ) + Math.cos( (i - j + 1) * Math.PI ) ) / 4
				xStabIntensity = Math.max(0, 1 - (1-xStabIntensity) * this.xStabWaveConcetrate)
			}
			const r = this.dataQubitWaveColor.r * dataQubitIntensity + this.xStabWaveColor.r * xStabIntensity + this.zStabWaveColor.r * zStabIntensity
			const g = this.dataQubitWaveColor.g * dataQubitIntensity + this.xStabWaveColor.g * xStabIntensity + this.zStabWaveColor.g * zStabIntensity
			const b = this.dataQubitWaveColor.b * dataQubitIntensity + this.xStabWaveColor.b * xStabIntensity + this.zStabWaveColor.b * zStabIntensity
			return [x, y, z, r, g, b]
		},
		generateDataQubits() {
			const qubits = []
			const array = []
			const bias = (this.L - 1) / 2
			for (let i=0; i < this.L; ++i) {
				const row = []
				for (let j=0; j < this.L; ++j) {
					const geometry = new THREE.SphereBufferGeometry( this.dataQubitSize, 48, 24 )
					const material = new THREE.MeshBasicMaterial( {
						color: this.dataQubitColor,
						envMap: this.three.scene.background,
						reflectivity: 0.5,
					} )
					const mesh = new THREE.Mesh( geometry, material )
					mesh.position.y = this.dataQubitYBias
					mesh.position.z = i - bias
					mesh.position.x = j - bias
					array.push(mesh)
					row.push(mesh)
				}
				qubits.push(row)
			}
			this.internals.dataQubits = qubits
			this.internals.dataQubitsArray = array
		},
		generateAncillaQubits() {
			const qubits = []
			const array = []
			const bias = (this.L - 1) / 2 + 0.5 // TODO
			for (let i=0; i <= this.L; ++i) {
				const row = []
				for (let j=0; j <= this.L; ++j) {
					const isZ = ((i + j) % 2) == 0
					let exist = true
					if (isZ && (j < 1 || j >= this.L)) exist = false
					if (!isZ && (i < 1 || i >= this.L)) exist = false
					if (!exist) {
						row.push(null)
						continue
					}
					const color = isZ ? this.zStabQubitColor : this.xStabQubitColor
					const geometry = new THREE.SphereBufferGeometry( this.dataQubitSize, 48, 24 )
					const material = new THREE.MeshBasicMaterial( {
						color: color,
						envMap: this.three.scene.background,
						reflectivity: 0.5,
					} )
					const mesh = new THREE.Mesh( geometry, material )
					mesh.position.y = this.dataQubitYBias - this.waveHeight
					mesh.position.z = i - bias
					mesh.position.x = j - bias
					array.push(mesh)
					row.push(mesh)
				}
				qubits.push(row)
			}
			this.internals.ancillaQubits = qubits
			this.internals.ancillaQubitsArray = array
		},

	},
}
</script>

<style scoped>

</style>
