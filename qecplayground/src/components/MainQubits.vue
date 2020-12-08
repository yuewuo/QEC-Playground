<template>
  <div class="main" id="main_qubits_container"></div>
</template>

<script>
import * as THREE from 'three'
import Stats from 'three/examples/jsm/libs/stats.module.js'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import axios from 'axios'

export default {
	name: 'MainQubits',
	props: {
		L: {
			type: Number,
			default: 5,
		},
		
		removeView: {  // used to debug other GUI, e.g. tutorial. This will remove all the initialization of 3D objects to speed up rendering
			type: Boolean,
			default: false,
		},
		decoderServerRootUrl: {
			type: String,
			default: () => "http://127.0.0.1:8066/"
		},
		dataErrorTopParameters: {  // list of [radius, angle, angleOffset, cyclePerSec] ...
			type: Array,
			default: () => [
				[0.15, Math.PI * 2 / 3, 0, 0.5],
				[0.175, Math.PI * 2 / 4, 0, 0.6],
				[0.2, Math.PI * 2 / 6, 0, 0.9]
			]
		},
		usePerspectiveCamera: {
			type: Boolean,
			default: true
		},
		dataQubitColor: {
			type: Object,
			default: () => new THREE.Color( 1, 0.65, 0 )
		},
		zStabQubitColor: {
			type: Object,
			default: () => new THREE.Color( 0, 0.75, 1 )
		},
		xStabQubitColor: {
			type: Object,
			default: () => new THREE.Color( 0, 0.8, 0 )
		},
		zStabErrorColor: {
			type: Object,
			default: () => new THREE.Color( 1, 0.15, 0.2 )
		},
		xStabErrorColor: {
			type: Object,
			default: () => new THREE.Color( 1, 0.16, 0 )
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
		defaultT: {  // the characteristic time of smooth animation
			type: Number,
			default: 0.1,
		},
		DataErrorOpacity: {
			type: Number,
			default: 0.9,
		},
		DataErrorTopOpacity: {
			type: Number,
			default: 0.5,
		},
		hoverAmplitude: {
			type: Number,
			default: 0.1
		},
		hoverDefaultT1: {
			type: Number,
			default: 0.3,
		},
		hoverDefaultT2: {
			type: Number,
			default: 0.07,
		},
		hoverColor: {
			type: Object,
			default: () => new THREE.Color( 1, 1, 1 )
		},
	},
	data() {
		return {
			three: { },  // save necessary THREE.js objects
			internals: { },  // internal data for control
			hoverDataQubit: null,  // null or [i, j, time]
			hoverAncillaQubit: null,  // null or [i, j, time]

			// controllable parameters for visualization
			dataQubitsDynamicYBias: [ ],  // [L][L] float numbers
			zDataQubitsErrors: [ ],  // [L][L] 0 ~ 1
			xDataQubitsErrors: [ ],  // [L][L] 0 ~ 1
			ancillaQubitsErrors: [ ],  // [L+1][L+1] 0 ~ 1
			hideZancilla: false,
			hideXancilla: false,
		}
	},
	mounted() {
		window.$mainQubits = this  // for fast debugging
		if (this.removeView) return

		const scene = new THREE.Scene()
		this.three.scene = scene
		this.three.clock = new THREE.Clock()
		this.three.clockAbs = new THREE.Clock()

		// create axios instance
		this.internals.axios = axios.create({
            baseURL: this.decoderServerRootUrl,
            method: 'get,post,put,patch,delete',
            headers: {
                "Access-Control-Allow-Origin": "*",
                "Access-Control-Allow-Headers": "Content-Type,X-Auth-Token",
                "Access-Control-Allow-Methods": "PUT,POST,GET",
            },
        })

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
		let container = document.getElementById('main_qubits_container')
		let orbitControl = new OrbitControls( camera, renderer.domElement )
		container.appendChild(renderer.domElement)

		// support for resize
		let that = this
		window.addEventListener( 'resize', () => {
			const windowWidth = window.innerWidth-this.panelWidth
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
			// scene.background = new THREE.Color(0xFFFFFF)
		}

		// add raycaster
		const raycaster = new THREE.Raycaster()
		this.three.raycaster = raycaster
		const mouse = new THREE.Vector2( 1, 1 )
		this.three.mouse = mouse
		document.addEventListener( 'mousemove', event => {
			event.preventDefault()
			const windowWidth = window.innerWidth-this.panelWidth
			const windowHeight = window.innerHeight
			mouse.x = ( event.clientX / windowWidth ) * 2 - 1
			mouse.y = - ( event.clientY / windowHeight ) * 2 + 1
		}, false )

		// create general materials to avoid recreating
		this.three.pointCloudMaterial = new THREE.PointsMaterial( { size: this.pointSize, vertexColors: true } )
		this.three.dataQubitGeometry = new THREE.SphereBufferGeometry( this.dataQubitSize, 48, 24 )
		this.three.ancillaQubitGeometry = new THREE.SphereBufferGeometry( this.dataQubitSize, 48, 24 )
		this.three.zDataErrorGeometry = new THREE.TorusGeometry( 0.1, 0.03, 32, 32 )
		this.three.zDataErrorGeometry.rotateY(Math.PI /2)
		this.three.xDataErrorGeometry = new THREE.TorusGeometry( 0.1, 0.03, 32, 32 )
		this.three.xDataErrorTopGeometries = []
		this.three.zDataErrorTopGeometries = []
		for (const obj of this.dataErrorTopParameters) {
			const [radius, angle, angleOffset] = obj
			const tube = 0.008
			const segments = 32
			const xGeometry = new THREE.TorusGeometry( radius, tube, segments, segments, angle )
			xGeometry.rotateZ(angleOffset)
			const zGeometry = new THREE.TorusGeometry( radius, tube, segments, segments, angle )
			zGeometry.rotateZ(angleOffset)
			zGeometry.rotateY(Math.PI / 2)
			this.three.xDataErrorTopGeometries.push(xGeometry)
			this.three.zDataErrorTopGeometries.push(zGeometry)
		}

		// update L
		this.onChangeL(this.L, null)

		// add stats if enabled
		if (this.enableStats) {
			this.three.stats = new Stats()
			container.appendChild( this.three.stats.dom )
		}

		// start rendering
		this.animate()

		// add mouse down event listener
		container.addEventListener( 'click', this.onMouseClicked, false )

	},
	methods: {
		async test() {

		},
		// run <500ms> later so that all the instances already have their texture mapping
		async paper_figure_prepare_white_background(sleep_ms = 500) {
			await this.sleep_ms(sleep_ms)
			this.three.scene.background = new THREE.Color( 1, 1, 1 )
		},
		async paper_figure_random_errors() {
			this.paper_figure_prepare_white_background()  // do not await
			this.L = 5
			await this.vue_next_tick()  // so that matrix are updated
			this.zDataQubitsErrors[0][0] = 1
			this.zDataQubitsErrors[3][2] = 1
			this.xDataQubitsErrors[3][2] = 1
			this.xDataQubitsErrors[1][4] = 1
			this.update_measurement()
			// reset camera
			const windowWidth = window.innerWidth - this.panelWidth
			const windowHeight = window.innerHeight
			const range = 3.5
			const camera = new THREE.OrthographicCamera( windowWidth / windowHeight * -range, windowWidth / windowHeight * range, range, -range, 0.1, 10000 )
			this.three.camera = camera
			camera.position.set( 0, 1, 0 )
			camera.lookAt( this.three.scene.position )
			camera.updateMatrix()
			this.three.scene.remove( this.internals.pointCloud )  // will cause exception when change L. OK because this is not interactive.
			// stop the error animation (but will stop at random phase...)
			for (let i=0; i<this.dataErrorTopParameters.length; ++i) this.dataErrorTopParameters[i][3] = 0
		},
		async paper_figure_single_stabilizer(x1, x2, x3, x4) {
			this.paper_figure_prepare_white_background()  // do not await
			this.L = 2  // only for plotting
			await this.vue_next_tick()  // so that matrix are updated
			this.three.camera.position.set( -1.8393678660619175, 2.0783821441393284, 0.2983748596088124 )
			this.three.camera.lookAt( this.three.scene.position )
			// set error
			this.xDataQubitsErrors[0][0] = x1
			this.xDataQubitsErrors[0][1] = x2
			this.xDataQubitsErrors[1][1] = x3
			this.xDataQubitsErrors[1][0] = x4
			this.ancillaQubitsErrors[1][1] = x1 ^ x2 ^ x3 ^ x4
			this.hideXancilla = true  // do not display them
			// stop the error animation (but will stop at random phase...)
			for (let i=0; i<this.dataErrorTopParameters.length; ++i) this.dataErrorTopParameters[i][3] = 0
		},
		async paper_figure_single_qubit_operator(x, z) {
			this.paper_figure_prepare_white_background()  // do not await
			this.L = 1
			await this.vue_next_tick()  // so that matrix are updated
			const initCameraRatio = this.L * 1.0
			// use $mainQubits.three.camera.position to see current position
			this.three.camera.position.set( -2 * initCameraRatio, 1 * initCameraRatio, 1 * initCameraRatio )
			this.xDataQubitsErrors[0][0] = x ? 1 : 0
			this.zDataQubitsErrors[0][0] = z ? 1 : 0
			// stop the error animation (but will stop at random phase...)
			for (let i=0; i<this.dataErrorTopParameters.length; ++i) this.dataErrorTopParameters[i][3] = 0
		},
		async sleep_ms(ms) {
			return new Promise((resolve, reject) => {
				setTimeout(() => { resolve() }, ms)
			})
		},
		async vue_next_tick() {
			let that = this
			await new Promise((resolve, reject) => {
				that.$nextTick(() => { resolve() })
			})
		},
		async get_correction(decoder="stupid_decoder", x_error, z_error) {
			x_error = x_error || this.xDataQubitsErrors
			z_error = z_error || this.zDataQubitsErrors
			let request_data = {
				L: this.L,
				x_error: x_error,
				z_error: z_error,
			}
			return (await this.internals.axios.request({
				url: "/" + decoder,
				method: "post",
				data: request_data,
			})).data
		},
		onMouseClicked() {
			if (this.hoverDataQubit != null) {
				let [i, j, absTime] = this.hoverDataQubit
				this.$emit('dataQubitClicked', [i, j, absTime])
			}
		},
		// from old change to new, after `T` seconds, the value difference would be 1/e of the original one
		smoothValue(val, old, delta, T=null, threshold=0.01) {
			if (T == null) T = this.defaultT
			if (Math.abs(val - old) <= threshold) return val
			return val + (old - val) * Math.exp( - delta / T )
		},
		smoothColor(val, old, delta, T=null, threshold=0.01) {
			const r = this.smoothValue(val.r, old.r, delta, T, threshold)
			const g = this.smoothValue(val.g, old.g, delta, T, threshold)
			const b = this.smoothValue(val.b, old.b, delta, T, threshold)
			return new THREE.Color( r, g, b )
		},
		hoverAnimation(delta, T1=null, T2=null) {  // 1 - exp(-delta/T1) * cos(delta/T2)
			if (T1 == null) T1 = this.hoverDefaultT1
			if (T2 == null) T2 = this.hoverDefaultT2
			return 1 - Math.exp( - delta / T1 ) * Math.cos( delta / T2 )
		},
		disposeOnRefresh(resource) {
			this.internals.disposable.push(resource)
			return resource
		},
		animate() {
			requestAnimationFrame( this.animate )  // call first
			const delta = this.three.clock.getDelta()
			const absTime = this.three.clockAbs.getElapsedTime()
			this.three.raycaster.setFromCamera( this.three.mouse, this.three.camera )
			if (this.internals.clickableGroup) {
				const intersection = this.three.raycaster.intersectObject( this.internals.clickableGroup, true )
				const hoverDataQubitLast = this.hoverDataQubit
				this.hoverDataQubit = null
				const hoverAncillaQubitLast = this.hoverAncillaQubit
				this.hoverAncillaQubit = null
				if ( intersection.length > 0 ) {
					const object = intersection[0].object
					this.internals.dataQubits.forEach((item, i) => {
						item.forEach((qubit, j) => {
							if (object == qubit) {
								if (hoverDataQubitLast && i == hoverDataQubitLast[0] && j == hoverDataQubitLast[1])
								this.hoverDataQubit = hoverDataQubitLast
								else this.hoverDataQubit = [i, j, absTime]
							}
						})
					})
					this.internals.ancillaQubits.forEach((item, i) => {
						item.forEach((qubit, j) => {
							if (object == qubit) {
								if (hoverAncillaQubitLast && i == hoverAncillaQubitLast[0] && j == hoverAncillaQubitLast[1])
									this.hoverAncillaQubit = hoverAncillaQubitLast
								else this.hoverAncillaQubit = [i, j, absTime]
							}
						})
					})
				}
			}
			for (let idx = 0; idx < this.dataErrorTopParameters.length; ++idx) {  // rotate error tops
				const cyclePerSec = this.dataErrorTopParameters[idx][3]
				this.three.xDataErrorTopGeometries[idx].rotateZ(Math.PI * 2 * cyclePerSec * delta)
				this.three.zDataErrorTopGeometries[idx].rotateX(Math.PI * 2 * cyclePerSec * delta)
			}
			for (let i = 0; i < this.internals.dataQubits.length; ++i) {
				for (let j = 0; j < this.internals.dataQubits[i].length; ++j) {
					let dataQubitTargetColor = this.dataQubitWaveColor
					let dataQubitYBias = 0
					if (this.hoverDataQubit != null && this.hoverDataQubit[0] == i && this.hoverDataQubit[1] == j) {
						dataQubitYBias = this.hoverAmplitude * this.hoverAnimation(absTime - this.hoverDataQubit[2])
						dataQubitTargetColor = this.hoverColor
					}
					const bias = this.smoothValue(this.dataQubitYBias + this.dataQubitsDynamicYBias[i][j] + dataQubitYBias
						, this.internals.dataQubits[i][j].position.y, delta)
					this.internals.dataQubits[i][j].position.y = bias
					this.internals.zDataErrors[i][j].position.y = bias
					this.internals.xDataErrors[i][j].position.y = bias
					this.internals.dataQubitsMaterials[i][j].color = this.smoothColor(dataQubitTargetColor
						, this.internals.dataQubitsMaterials[i][j].color, delta)
					const zDataError = this.zDataQubitsErrors[i][j]
					this.internals.zDataErrorMaterials[i][j].opacity = this.smoothValue(zDataError * this.DataErrorOpacity
						, this.internals.zDataErrorMaterials[i][j].opacity, delta)
					this.internals.zDataErrorTopMaterials[i][j].opacity = this.smoothValue(zDataError * this.DataErrorTopOpacity
						, this.internals.zDataErrorTopMaterials[i][j].opacity, delta)
					const xDataError = this.xDataQubitsErrors[i][j]
					this.internals.xDataErrorMaterials[i][j].opacity = this.smoothValue(xDataError * this.DataErrorOpacity
						, this.internals.xDataErrorMaterials[i][j].opacity, delta)
					this.internals.xDataErrorTopMaterials[i][j].opacity = this.smoothValue(xDataError * this.DataErrorTopOpacity
						, this.internals.xDataErrorTopMaterials[i][j].opacity, delta)
				}
			}
			for (let i=0; i <= this.L; ++i) {
				for (let j=0; j <= this.L; ++j) {
					const isZ = ((i + j) % 2) == 0
					let exist = true
					if (isZ && (j < 1 || j >= this.L)) exist = false
					if (!isZ && (i < 1 || i >= this.L)) exist = false
					if (!exist) continue
					let targetColor = isZ ? this.zStabQubitColor : this.xStabQubitColor
					if (this.ancillaQubitsErrors[i][j]) targetColor = isZ ? this.zStabErrorColor : this.xStabErrorColor
					this.internals.ancillaQubitsMaterials[i][j].color = this.smoothColor(targetColor
						, this.internals.ancillaQubitsMaterials[i][j].color, delta)
					this.internals.ancillaQubitsMaterials[i][j].opacity = this.smoothValue(((isZ && !this.hideZancilla) || (!isZ && !this.hideXancilla)) ? 1. : 0.
						, this.internals.ancillaQubitsMaterials[i][j].opacity, delta)
				}
			}
			for (let i = 0; i < this.internals.ancillaQubits.length; ++i) {
				for (let j = 0; j < this.internals.ancillaQubits[i].length; ++j) {
					let qubit = this.internals.ancillaQubits[i][j]
					if (qubit) qubit.position.y = this.dataQubitYBias - this.waveHeight + this.ancillaQubitsDynamicYBias[i][j]
				}
			}
			if (this.three.stats) this.three.stats.update()  // update stats if exists
			this.three.renderer.render( this.three.scene, this.three.camera )
		},
		generatePointCloud() {
			const geometry = this.disposeOnRefresh(new THREE.BufferGeometry())
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
			this.internals.pointCloud = new THREE.Points( geometry, this.three.pointCloudMaterial )
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
		copySquareArray(array) {
			return this.makeSquareArray(array.length, (i,j) => array[i][j])
		},
		generateDataQubits() {
			const qubits = []
			const array = []
			const bias = (this.L - 1) / 2
			this.dataQubitsDynamicYBias = this.makeSquareArray(this.L)
			this.zDataQubitsErrors = this.makeSquareArray(this.L)
			this.xDataQubitsErrors = this.makeSquareArray(this.L)
			const disposeOnRefresh = this.disposeOnRefresh
			this.internals.dataQubitsMaterials = this.makeSquareArray(this.L, (() => disposeOnRefresh(new THREE.MeshBasicMaterial( {
				color: this.dataQubitColor,
				envMap: this.three.scene.background,
				reflectivity: 0.5,
			} ))))
			for (let i=0; i < this.L; ++i) {
				const row = []
				for (let j=0; j < this.L; ++j) {
					const mesh = new THREE.Mesh( this.three.dataQubitGeometry, this.internals.dataQubitsMaterials[i][j] )
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
			const bias = (this.L - 1) / 2 + 0.5
			this.ancillaQubitsDynamicYBias = this.makeSquareArray(this.L + 1)
			this.ancillaQubitsErrors = this.makeSquareArray(this.L + 1)
			const disposeOnRefresh = this.disposeOnRefresh
			this.internals.ancillaQubitsMaterials = this.makeSquareArray(this.L + 1, ((i,j) => disposeOnRefresh(new THREE.MeshBasicMaterial( {
				color: ((i + j) % 2) == 0 ? this.zStabQubitColor : this.xStabQubitColor,
				envMap: this.three.scene.background,
				reflectivity: 0.5,
				transparent: true,  // allow opacity,
				opacity: 1,
			} ))))
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
					const mesh = new THREE.Mesh( this.three.ancillaQubitGeometry, this.internals.ancillaQubitsMaterials[i][j] )
					mesh.position.y = this.dataQubitYBias - this.waveHeight
					mesh.position.z = i - bias
					mesh.position.x = j - bias
					mesh.rotation.set(0, 0, isZ ? 0 : Math.PI / 2)
					array.push(mesh)
					row.push(mesh)
				}
				qubits.push(row)
			}
			this.internals.ancillaQubits = qubits
			this.internals.ancillaQubitsArray = array
		},
		generateDataQubitsErrors() {
			const zErrors = []  // each element in this is a group of mesh
			const xErrors = []
			const array = []
			const bias = (this.L - 1) / 2
			const disposeOnRefresh = this.disposeOnRefresh
			this.internals.zDataErrorMaterials = this.makeSquareArray(this.L, (() => disposeOnRefresh(new THREE.MeshPhongMaterial( {
				transparent: true,
				opacity: 0,
				emissive: this.zStabQubitColor,
			} ))))
			this.internals.xDataErrorMaterials = this.makeSquareArray(this.L, (() => disposeOnRefresh(new THREE.MeshPhongMaterial( {
				transparent: true,
				opacity: 0,
				emissive: this.xStabQubitColor,
			} ))))
			const dataErrorTopMaterialsGenerator = (() => disposeOnRefresh(new THREE.MeshPhongMaterial( {
				transparent: true,
				opacity: 0,
				emissive: 0xFF0000,
			} )))
			this.internals.zDataErrorTopMaterials = this.makeSquareArray(this.L, dataErrorTopMaterialsGenerator)
			this.internals.xDataErrorTopMaterials = this.makeSquareArray(this.L, dataErrorTopMaterialsGenerator)
			for (let i=0; i < this.L; ++i) {
				const zRow = []
				const xRow = []
				for (let j=0; j < this.L; ++j) {
					for (let isZ = 0; isZ < 2; ++isZ) {
						const group = new THREE.Group()
						const dataErrorMaterial = (isZ ? this.internals.zDataErrorMaterials : this.internals.xDataErrorMaterials)[i][j]
						const dataErrorTopMaterial = (isZ ? this.internals.zDataErrorTopMaterials : this.internals.xDataErrorTopMaterials)[i][j]
						for (let k = -1; k < 2; k += 2) {
							const z = !isZ * k * this.dataQubitSize * 1.2 + i - bias
							const x = isZ * k * this.dataQubitSize * 1.2 + j - bias
							const geometry = isZ ? this.three.zDataErrorGeometry : this.three.xDataErrorGeometry
							const mesh = new THREE.Mesh( geometry, dataErrorMaterial )
							mesh.position.set(x, 0, z)
							group.add(mesh)
							// add error tops for fancy visualization
							for (const geometry of (isZ ? this.three.zDataErrorTopGeometries : this.three.xDataErrorTopGeometries)) {
								const mesh = new THREE.Mesh( geometry, dataErrorTopMaterial )
								mesh.position.set(x, 0, z)
								mesh.rotateY((k + 1) / 2 * Math.PI)
								group.add(mesh)
							}
						}
						group.position.y = this.dataQubitYBias
						const row = isZ ? zRow : xRow
						row.push(group)
						array.push(group)
					}
				}
				zErrors.push(zRow)
				xErrors.push(xRow)
			}
			this.internals.zDataErrors = zErrors
			this.internals.xDataErrors = xErrors
			this.internals.dataErrorArray = array
		},
		onChangeL(val, old) {
			if (old != null) {  // destroy things
				for (const disposable of this.internals.disposable) disposable.dispose()
				this.three.scene.remove( this.internals.pointCloud )
				this.three.scene.remove( this.internals.clickableGroup )
				for (const obj of this.internals.dataErrorArray) this.three.scene.remove( obj )
				// return   // test the functionality of remove (check if all objects disappear when L changes)
			}
			this.internals.disposable = []
			// add qubits
			this.generatePointCloud()
			this.three.scene.add( this.internals.pointCloud )
			this.generateDataQubits()
			this.generateAncillaQubits()
			const clickableGroup = new THREE.Group()
			for (const obj of this.internals.dataQubitsArray) clickableGroup.add(obj)
			for (const obj of this.internals.ancillaQubitsArray) clickableGroup.add(obj)
			this.three.scene.add( clickableGroup )
			this.internals.clickableGroup = clickableGroup
			this.generateDataQubitsErrors()
			for (const obj of this.internals.dataErrorArray) this.three.scene.add(obj)
			const initCameraRatio = this.L * 0.5
			this.three.camera.position.set( -2 * initCameraRatio, 1 * initCameraRatio, 1 * initCameraRatio )
			this.three.camera.lookAt( this.three.scene.position )
		},
		update_measurement() {  // measure `this.zDataQubitsErrors` and `this.xDataQubitsErrors` then update `this.ancillaQubitsErrors`
			for (let i=0; i<=this.L; ++i) {
				for (let j=0; j<=this.L; ++j) {
					if (j != 0 && j != this.L && (i+j)%2 == 0) {  // Z stabilizer only when i+j is even
						// XOR a(i-1,j-1), b(i-1,j), c(i,j-1), d(i,j) if exist
						let i_minus_exists = i > 0
						let i_exists = i < this.L
						let result = 0
						if (i_minus_exists) {
							result ^= this.xDataQubitsErrors[i-1][j-1] ^ this.xDataQubitsErrors[i-1][j]
						}
						if (i_exists) {
							result ^= this.xDataQubitsErrors[i][j-1] ^ this.xDataQubitsErrors[i][j]
						}
						this.ancillaQubitsErrors[i][j] = result
					}
					if (i != 0 && i != this.L && (i+j)%2 == 1) {  // X stabilizer only when i+j is odd
						// XOR a(i-1,j-1), b(i-1,j), c(i,j-1), d(i,j) if exist
						let j_minus_exists = j > 0
						let j_exists = j < this.L
						let result = 0
						if (j_minus_exists) {
							result ^= this.zDataQubitsErrors[i-1][j-1] ^ this.zDataQubitsErrors[i][j-1];
						}
						if (j_exists) {
							result ^= this.zDataQubitsErrors[i-1][j] ^ this.zDataQubitsErrors[i][j];
						}
						this.ancillaQubitsErrors[i][j] = result
					}
				}
			}
		},
	},
	watch: {
		L(val, old) {
			this.onChangeL(val, old)
		},
	},
}
</script>

<style scoped>

.main {
	background: black;
}

</style>
