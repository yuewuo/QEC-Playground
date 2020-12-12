<template>
  <div class="main" id="fault_tolerant_view_container"></div>
</template>

<script>
import * as THREE from 'three'
import Stats from 'three/examples/jsm/libs/stats.module.js'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'

// use this function to restriction modification to constants
function readonly(target, keys) {
    var _cloned = {}
    function makeReadOnlyProperty(cloned, obj, prop) {
        Object.defineProperty(cloned, prop, {
            set: function() {
                throw {
                    name: 'UnableRewriteException',
                    message: 'original cannot be rewrite'
                }
            },
            get: function() {
                return obj[prop]
            },
            enumerable: true
        })
    }
    for (var prop in target) {
        makeReadOnlyProperty(_cloned, target, prop)
    }
    return _cloned
}

export default {
	name: 'FaultTolerantView',
	props: {
		L: {
			type: Number,
			default: 5,
		},
		T: {
			type: Number,
			default: 4,
		},
		
		
		panelWidth: {
			type: Number,
			default: 480
        },
        errorModel: {
            type: String,
            default: "depolarizing",
        },
        depolarErrorRate: {  // used when errorModel = "depolarizing", (1-3p) + pX + pZ + pY, px = 2p, pz = 2p
            type: Number,
            default: 0.001,
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
			internals: { bias: { x:0, y:0, z:0 } },  // internal data

            // controllable parameters for visualization
            snapshot: null,  // [t][i][j]
            constants: null, // { QTYPE (qubit type), NTYPE (node type), etc. }
            show_data_qubit: true,
            show_X_ancilla: true,
            show_Z_ancilla: true,
            show_vertical_line: true,
            show_initialization: true,
            show_CX_gates: true,
            show_X_edges: true,
            show_Z_edges: true,
		}
	},
	mounted() {
        this.build_constants()
        window.THREE = THREE
		window.$ftview = this  // for fast debugging

		const scene = new THREE.Scene()
		this.three.scene = scene
		this.three.clock = new THREE.Clock()
		this.three.clockAbs = new THREE.Clock()

		// add camera and renderer
		const windowWidth = window.innerWidth - this.panelWidth
        const windowHeight = window.innerHeight
        const camera = new THREE.PerspectiveCamera( 75, windowWidth / window.innerHeight, 0.1, 10000 )
        // const camera = new THREE.OrthographicCamera( windowWidth / windowHeight * -3, windowWidth / windowHeight * 3, 3, -3, 0.1, 10000 )
		this.three.camera = camera
		const initCameraRatio = this.L * 0.8
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
        
        this.create_static_resources()
        this.swap_snapshot(this.build_standard_planar_code_snapshot())
        // this.swap_snapshot(this.build_rotated_planar_code())

		// start rendering
        this.animate()
        
        this.test()

	},
	methods: {
		async test() {
            this.paper_figure_Z_stabilizer_connection()
        },
        async paper_figure_Z_stabilizer_connection() {
            this.show_data_qubit = false
            this.show_X_ancilla = false
            this.show_X_ancilla = false
            this.show_vertical_line = false
            this.show_initialization = false
            this.show_CX_gates = false
            this.show_X_edges = false
        },
        async paper_figure_single_error_two_syndrome() {
            this.snapshot[3][1][2].error = this.constants.ETYPE.X
            this.compute_propagated_error()
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
        build_constants() {
            this.constants = readonly({
                QTYPE: readonly({  // qubit type
                    DATA: 0,
                    X: 1,
                    Z: 2,
                }),
                NTYPE: readonly({  // node type, correspond to the nodes in time sequence fiure with detailed gate operations
                    INITIALIZATION: 0,
                    CONTROL: 1,
                    TARGET: 2,
                    MEASUREMENT: 3,
                    NONE: 4,
                    NONE_WITH_DATA_QUBIT: 5,  // for purpose of plotting data qubits
                }),
                ETYPE: readonly({  // node type, correspond to the nodes in time sequence fiure with detailed gate operations
                    I: 0,  // no error
                    X: 1,  // Pauli X error
                    Z: 2,  // Pauli Z error
                    Y: 3,  // both Pauli X and Z error
                }),
                VERTICAL_INTERVAL: 0.333,
            })
        },
		animate() {
			requestAnimationFrame( this.animate )  // call first
			const delta = this.three.clock.getDelta()
			const absTime = this.three.clockAbs.getElapsedTime()
			if (this.three.stats) this.three.stats.update()  // update stats if exists
			this.three.renderer.render( this.three.scene, this.three.camera )
        },
        reset_snapshot() {
            // TODO: implement resource destroy if structure are meant to be changed dynamically
            this.snapshot = null
        },
        swap_snapshot(snapshot) {
            this.reset_snapshot()
            this.snapshot = snapshot
            this.build_graph_given_error_rate()
            this.establish_snapshot()
        },
        build_code_in_standard_planar_code(filter=((i,j)=>true)) {  // filter determines whether there is a qubit at [t][i][j]
            console.assert(this.T >= 1, "T should be at least 1, 1 is for perfect measurement condition")
            const width = 2 * this.L - 1
            const height = this.T * 6 + 1
            let snapshot = []
            for (let t=0; t<height; ++t) {
                let snapshot_row_0 = []
                for (let i=0; i<width; ++i) {
                    let snapshot_row_1 = []
                    for (let j=0; j<width; ++j) {
                        if (filter(i,j)) {  // if here exists a qubit (either data qubit or ancilla qubit)
                            const stage = (t+6-1) % 6  // 0: preparation, 1,2,3,4: CNOT gate, 5: measurement
                            const is_data_qubit = (i+j)%2 == 0 
                            const q_type = is_data_qubit ? this.constants.QTYPE.DATA : (i % 2 == 0 ? this.constants.QTYPE.Z : this.constants.QTYPE.X)
                            let n_type = -1
                            let connection = null  // { t, i, j, }
                            switch (stage) {
                                case 0:
                                    n_type = is_data_qubit ? this.constants.NTYPE.NONE : this.constants.NTYPE.INITIALIZATION
                                    break
                                case 1:
                                    if (is_data_qubit) {
                                        if (i+1 < width && filter(i+1, j)) {
                                            if (j % 2 == 0) n_type = this.constants.NTYPE.TARGET
                                            else n_type = this.constants.NTYPE.CONTROL
                                            connection = { i:i+1, j, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    } else {
                                        if (i-1 >= 0 && filter(i-1, j)) {
                                            if (j % 2 == 0) n_type = this.constants.NTYPE.CONTROL
                                            else n_type = this.constants.NTYPE.TARGET
                                            connection = { i:i-1, j, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    }
                                    break
                                case 2:
                                    if (is_data_qubit) {
                                        if (j+1 < width && filter(i, j+1)) {
                                            if (i % 2 == 0) n_type = this.constants.NTYPE.CONTROL
                                            else n_type = this.constants.NTYPE.TARGET
                                            connection = { i, j:j+1, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    } else {
                                        if (j-1 >= 0 && filter(i, j-1)) {
                                            if (i % 2 == 0) n_type = this.constants.NTYPE.TARGET
                                            else n_type = this.constants.NTYPE.CONTROL
                                            connection = { i, j:j-1, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    }
                                    break
                                case 3:
                                    if (is_data_qubit) {
                                        if (j-1 >= 0 && filter(i, j-1)) {
                                            if (i % 2 == 0) n_type = this.constants.NTYPE.CONTROL
                                            else n_type = this.constants.NTYPE.TARGET
                                            connection = { i, j:j-1, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    } else {
                                        if (j+1 < width && filter(i, j+1)) {
                                            if (i % 2 == 0) n_type = this.constants.NTYPE.TARGET
                                            else n_type = this.constants.NTYPE.CONTROL
                                            connection = { i, j:j+1, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    }
                                    break
                                case 4:
                                    if (is_data_qubit) {
                                        if (i-1 >= 0 && filter(i-1, j)) {
                                            if (j % 2 == 0) n_type = this.constants.NTYPE.TARGET
                                            else n_type = this.constants.NTYPE.CONTROL
                                            connection = { i:i-1, j, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    } else {
                                        if (i+1 < width && filter(i+1, j)) {
                                            if (j % 2 == 0) n_type = this.constants.NTYPE.CONTROL
                                            else n_type = this.constants.NTYPE.TARGET
                                            connection = { i:i+1, j, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    }
                                    break
                                case 5:
                                    n_type = is_data_qubit ? this.constants.NTYPE.NONE_WITH_DATA_QUBIT : this.constants.NTYPE.MEASUREMENT
                                    break
                            }
                            let qubit = {
                                t, i, j,
                                connection,
                                n_type,
                                q_type,
                                error: this.constants.ETYPE.I,  // an error happening from now to next
                                propagated: this.constants.ETYPE.I,  // propagted error till now
                            }
                            if (this.errorModel == "depolarizing") {
                                qubit.px = 2 * this.depolarErrorRate  // X error rate
                                qubit.pz = 2 * this.depolarErrorRate  // Z error rate
                            }
                            snapshot_row_1.push(qubit)
                        } else {
                            snapshot_row_1.push(null)
                        }
                    }
                    snapshot_row_0.push(snapshot_row_1)
                }
                snapshot.push(snapshot_row_0)
            }
            return snapshot
        },
        build_standard_planar_code_snapshot() {
            let snapshot = this.build_code_in_standard_planar_code()
            // add boundary information (only add possible boundaries. exact boundary will be added `p` after building the graph)
            for (let t=6; t < snapshot.length; t+=6) {
                for (let i=0; i < snapshot[t].length; ++i) {
                    for (let j=0; j < snapshot[t][i].length; ++j) {
                        let node = snapshot[t][i][j]
                        if (!node) continue
                        if (node.n_type == this.constants.NTYPE.MEASUREMENT) {
                            let bt = t
                            let bi = i
                            let bj = j
                            if (t == snapshot.length - 1) bt += 6
                            else {
                                if (i == 1) bi -= 2
                                if (i == snapshot[t].length - 2) bi += 2
                                if (j == 1) bj -= 2
                                if (j == snapshot[t][i].length - 2) bj += 2
                            }
                            if (bi != i || bj != j || bt != t) {
                                node.boundary = {
                                    t: bt,
                                    i: bi,
                                    j: bj,
                                }
                            }
                        }
                    }
                }
            }
            return snapshot
        },
        build_rotated_planar_code() {
            console.assert(this.L % 2 == 1, "L should be odd")  // odd ensures a balanced x and z correction
            const middle = this.L - 1
            const constants = this.constants
            function filter(i, j) {
                const distance = Math.abs(i - middle) + Math.abs(j - middle)
                if (distance <= middle) return true
                if ((i+j)%2 == 0) return false  // data qubit
                const q_type = i % 2 == 0 ? constants.QTYPE.Z : constants.QTYPE.X
                if (q_type == constants.QTYPE.Z && (i-middle)*(j-middle) > 0) return distance <= middle + 1
                if (q_type == constants.QTYPE.X && (i-middle)*(j-middle) < 0) return distance <= middle + 1
            }
            let snapshot = this.build_code_in_standard_planar_code(filter)
            // add boundary information (only add possible boundaries. exact boundary will be added `p` after building the graph)
            for (let t=6; t < snapshot.length; t+=6) {
                for (let i=0; i < snapshot[t].length; ++i) {
                    for (let j=0; j < snapshot[t][i].length; ++j) {
                        let node = snapshot[t][i][j]
                        if (!node) continue
                        if (node.n_type == this.constants.NTYPE.MEASUREMENT) {
                            let bt = t
                            let bi = i
                            let bj = j
                            const distance = Math.abs(i - middle) + Math.abs(j - middle)
                            if (t == snapshot.length - 1) bt += 6
                            else if (distance >= middle - 3) {
                                const q_type = i % 2 == 0 ? this.constants.QTYPE.Z : this.constants.QTYPE.X
                                if (q_type == this.constants.QTYPE.Z) {
                                    if (i > j) {
                                        bi += 2
                                        bj -= 2
                                    } else {
                                        bi -= 2
                                        bj += 2
                                    }
                                } else {
                                    if (i + j > 2 * middle) {
                                        bi += 2
                                        bj += 2
                                    } else {
                                        bi -= 2
                                        bj -= 2
                                    }
                                }
                            }
                            if (bi != i || bj != j || bt != t) {
                                node.boundary = {
                                    t: bt,
                                    i: bi,
                                    j: bj,
                                }
                            }
                        }
                    }
                }
            }
            return snapshot
        },
        error_multiply(err1, err2) {  // return err1.err2
            if (err1 == this.constants.ETYPE.I) return err2
            if (err2 == this.constants.ETYPE.I) return err1
            if (err1 == this.constants.ETYPE.X && err2 == this.constants.ETYPE.X) return this.constants.ETYPE.I
            if (err1 == this.constants.ETYPE.X && err2 == this.constants.ETYPE.Z) return this.constants.ETYPE.Y
            if (err1 == this.constants.ETYPE.X && err2 == this.constants.ETYPE.Y) return this.constants.ETYPE.Z
            if (err1 == this.constants.ETYPE.Z && err2 == this.constants.ETYPE.X) return this.constants.ETYPE.Y
            if (err1 == this.constants.ETYPE.Z && err2 == this.constants.ETYPE.Z) return this.constants.ETYPE.I
            if (err1 == this.constants.ETYPE.Z && err2 == this.constants.ETYPE.Y) return this.constants.ETYPE.X
            if (err1 == this.constants.ETYPE.Y && err2 == this.constants.ETYPE.X) return this.constants.ETYPE.Z
            if (err1 == this.constants.ETYPE.Y && err2 == this.constants.ETYPE.Z) return this.constants.ETYPE.X
            if (err1 == this.constants.ETYPE.Y && err2 == this.constants.ETYPE.Y) return this.constants.ETYPE.I
        },
        compute_propagated_error(update_view=true) {
            // careful: t=0 will remain propagated error, others will be recomputed
            for (let t=1; t < this.snapshot.length; ++t) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        if (!this.snapshot[t][i][j]) continue
                        this.snapshot[t][i][j].propagated = this.constants.ETYPE.I
                    }
                }
            }
            for (let t=0; t < this.snapshot.length-1; ++t) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        const node = this.snapshot[t][i][j]
                        if (!node) continue
                        if (node.n_type == this.constants.NTYPE.INITIALIZATION) {
                            node.propagated = this.constants.ETYPE.I  // no error when initialized
                        }
                        // error will definitely propagated to itself
                        const direct_error = this.error_multiply(node.error, node.propagated)
                        this.snapshot[t+1][i][j].propagated = this.error_multiply(direct_error, this.snapshot[t+1][i][j].propagated)
                        // but sometimes it also propagated to other qubits through CX gate
                        if (node.n_type == this.constants.NTYPE.CONTROL) {
                            if (node.propagated == this.constants.ETYPE.X || node.propagated == this.constants.ETYPE.Y) {
                                const peer_node = this.snapshot[t+1][node.connection.i][node.connection.j]
                                peer_node.propagated = this.error_multiply(this.constants.ETYPE.X, peer_node.propagated)
                            }
                        }
                        if (node.n_type == this.constants.NTYPE.TARGET) {
                            if (node.propagated == this.constants.ETYPE.Z || node.propagated == this.constants.ETYPE.Y) {
                                const peer_node = this.snapshot[t+1][node.connection.i][node.connection.j]
                                peer_node.propagated = this.error_multiply(this.constants.ETYPE.Z, peer_node.propagated)
                            }
                        }
                    }
                }
            }
            if (update_view) {
                for (let t=1; t < this.snapshot.length; ++t) {  // t=1 necessary, do not update the lowest layer
                    for (let i=0; i < this.snapshot[t].length; ++i) {
                        for (let j=0; j < this.snapshot[t][i].length; ++j) {
                            const node = this.snapshot[t][i][j]
                            if (!node) continue
                            if (node.n_type == this.constants.NTYPE.MEASUREMENT) {
                                if (node.q_type == this.constants.QTYPE.Z) {
                                    let this_result = node.propagated == this.constants.ETYPE.I || node.propagated == this.constants.ETYPE.Z
                                    const last_node = this.snapshot[t-6][i][j]
                                    let last_result = last_node.propagated == this.constants.ETYPE.I || last_node.propagated == this.constants.ETYPE.Z
                                    if (this_result != last_result) {
                                        node.mesh.material.color = this.three.measurement_node_color_error
                                    } else node.mesh.material.color = this.three.initialization_node_color_Z
                                } else {
                                    let this_result = node.propagated == this.constants.ETYPE.I || node.propagated == this.constants.ETYPE.X
                                    const last_node = this.snapshot[t-6][i][j]
                                    let last_result = last_node.propagated == this.constants.ETYPE.I || last_node.propagated == this.constants.ETYPE.X
                                    if (this_result != last_result) {
                                        node.mesh.material.color = this.three.measurement_node_color_error
                                    } else node.mesh.material.color = this.three.initialization_node_color_X
                                }
                            }
                            if (t > 0) {
                                const vertical = this.snapshot[t][i][j].vertical
                                if (node.propagated == this.constants.ETYPE.I) vertical.material.color = this.three.vertical_line_color
                                else vertical.material.color = this.three.measurement_node_color_error
                            }
                        }
                    }
                }
            }
        },
        position_middle_set_bias() {
            const [x, y, z] = this.position(0,0,0)
            let mins = [x, y, z]
            let maxs = [x, y, z]
            let search = [[this.snapshot.length-1,0,0], [0,this.snapshot[0].length-1,0], [0,0,this.snapshot[0][0].length-1]]
            for (let val of search) {
                let pos = this.position(val[0], val[1], val[2])
                for (let i=0; i<3; ++i) {
                    if (pos[i] < mins[i]) mins[i] = pos[i]
                    if (pos[i] > maxs[i]) maxs[i] = pos[i]
                }
            }
            this.internals.bias.x = -0.5 * (maxs[0] - mins[0])
            this.internals.bias.y = -0.5 * (maxs[1] - mins[1])
            this.internals.bias.z = -0.5 * (maxs[2] - mins[2])
        },
        no_bias_position(t, i, j) {  // requires = 0 when t=i=j=0
            return [j, t * this.constants.VERTICAL_INTERVAL, i]
        },
        position(t, i, j) {
            let [x, y, z] = this.no_bias_position(t, i, j)
            return [x + this.internals.bias.x, y + this.internals.bias.y, z + this.internals.bias.z]
        },
        create_static_resources() {
            this.three.default_sphere = new THREE.SphereBufferGeometry( 0.2, 48, 24 )
            this.three.initialization_node_geometry = new THREE.ConeBufferGeometry( 0.1, 0.15, 32 )
            this.three.initialization_node_color_Z = new THREE.Color( 0, 0.75, 1 )
            this.three.initialization_node_color_X = new THREE.Color( 0, 0.8, 0 )
            this.three.measurement_node_geometry = new THREE.SphereBufferGeometry( 0.2, 48, 24 )
            this.three.measurement_node_color_Z = new THREE.Color( 0, 0.75, 1 )
            this.three.measurement_node_color_X = new THREE.Color( 0, 0.8, 0 )
            this.three.measurement_node_color_error = new THREE.Color( 'red' )
            this.three.data_node_geometry = new THREE.SphereBufferGeometry( 0.2, 48, 24 )
            this.three.data_node_color = new THREE.Color( 1, 0.65, 0 )
            const vertical_radius = 0.01
            this.three.vertical_line_geometry = new THREE.CylinderBufferGeometry( vertical_radius, vertical_radius, this.constants.VERTICAL_INTERVAL, 6 )
            this.three.vertical_line_geometry.translate(0, - 0.5 * this.constants.VERTICAL_INTERVAL, 0)
            this.three.vertical_line_color = new THREE.Color( 'black' )
            const control_radius = 0.15
            const control_tube = 0.005
            this.three.CX_target_geometries = [
                new THREE.TorusBufferGeometry( control_radius, control_tube, 16, 32 ),
                new THREE.CylinderBufferGeometry( control_tube, control_tube, 2 * control_radius, 6 ),
                new THREE.CylinderBufferGeometry( control_tube, control_tube, 2 * control_radius, 6 ),
            ]
            this.three.CX_target_geometries[0].rotateX(Math.PI / 2)
            this.three.CX_target_geometries[1].rotateX(Math.PI / 2)
            this.three.CX_target_geometries[2].rotateZ(Math.PI / 2)
            this.three.CX_target_color = new THREE.Color( 'black' )
            this.three.CX_link_geometry = new THREE.CylinderBufferGeometry( control_tube, control_tube, 1, 6 )
            this.three.CX_link_geometry.translate(0, 0.5, 0)
            this.three.CX_link_color = new THREE.Color( 'black' )
            this.three.CX_control_geometry = new THREE.SphereBufferGeometry( 0.03, 12, 6 )
            this.three.CX_control_color = new THREE.Color( 'black' )
            const edge_default_radius = 0.05
            this.three.edge_geometry = new THREE.CylinderBufferGeometry( edge_default_radius, edge_default_radius, 1, 6 ),
            this.three.edge_geometry.translate(0, 0.5, 0)
            this.three.edge_color = new THREE.Color( 'black' )
        },
        establish_snapshot() {
            // position all object in the middle
            this.position_middle_set_bias()
            // add objects
            for (let t=0; t < this.snapshot.length; ++t) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        let node = this.snapshot[t][i][j]
                        if (node != null) {
                            const [x, y, z] = this.position(t, i, j)
                            if (node.n_type == this.constants.NTYPE.INITIALIZATION) {
                                const color = node.q_type == this.constants.QTYPE.Z ? this.three.initialization_node_color_Z : this.three.initialization_node_color_X
                                node.mesh = new THREE.Mesh(this.three.initialization_node_geometry, new THREE.MeshBasicMaterial({
                                    color,
                                }))
                                node.mesh.position.set(x, y, z)
                                this.three.scene.add(node.mesh)
                            }
                            if (node.n_type == this.constants.NTYPE.MEASUREMENT) {
                                const color = node.q_type == this.constants.QTYPE.Z ? this.three.measurement_node_color_Z : this.three.measurement_node_color_X
                                node.mesh = new THREE.Mesh(this.three.measurement_node_geometry, new THREE.MeshBasicMaterial({
                                    color,
                                    envMap: this.three.scene.background,
                                    reflectivity: 0.5,
                                }))
                                node.mesh.position.set(x, y, z)
                                this.three.scene.add(node.mesh)
                            }
                            if (node.n_type == this.constants.NTYPE.NONE_WITH_DATA_QUBIT) {
                                node.mesh = new THREE.Mesh(this.three.data_node_geometry, new THREE.MeshBasicMaterial({
                                    color: this.three.data_node_color,
                                    envMap: this.three.scene.background,
                                    reflectivity: 0.5,
                                }))
                                node.mesh.position.set(x, y, z)
                                this.three.scene.add(node.mesh)
                            }
                            if (node.n_type == this.constants.NTYPE.TARGET) {
                                node.mesh = []
                                for (let k=0; k < this.three.CX_target_geometries.length; ++k) {
                                    const geometry = this.three.CX_target_geometries[k]
                                    let mesh = new THREE.Mesh(geometry, new THREE.MeshBasicMaterial({
                                        color: this.three.CX_target_color,
                                    }))
                                    node.mesh.push(mesh)
                                    mesh.position.set(x, y, z)
                                    this.three.scene.add(mesh)
                                }
                                // also add CX link here
                                let mesh = new THREE.Mesh(this.three.CX_link_geometry, new THREE.MeshBasicMaterial({
                                    color: this.three.CX_link_color,
                                }))
                                if (node.connection.i == i+1) {
                                    mesh.rotateX(Math.PI / 2)
                                }
                                if (node.connection.i == i-1) {
                                    mesh.rotateX(-Math.PI / 2)
                                }
                                if (node.connection.j == j+1) {
                                    mesh.rotateZ(-Math.PI / 2)
                                }
                                if (node.connection.j == j-1) {
                                    mesh.rotateZ(Math.PI / 2)
                                }
                                node.mesh.push(mesh)
                                mesh.position.set(x, y, z)
                                this.three.scene.add(mesh)
                            }
                            if (node.n_type == this.constants.NTYPE.CONTROL) {
                                node.mesh = new THREE.Mesh(this.three.CX_control_geometry, new THREE.MeshBasicMaterial({
                                    color: this.three.CX_control_color,
                                }))
                                node.mesh.position.set(x, y, z)
                                this.three.scene.add(node.mesh)
                            }
                            // draw vertical line
                            if (t > 0) {
                                node.vertical = new THREE.Mesh(this.three.vertical_line_geometry, new THREE.MeshBasicMaterial({
                                    color: this.three.vertical_line_color,
                                }))
                                node.vertical.position.set(x, y, z)
                                this.three.scene.add(node.vertical)
                            }
                            // draw edges (automatically built graph)
                            const generate_half_edge_mesh = function(t, i, j, pt, pi, pj) {
                                const mesh = new THREE.Mesh(this.three.edge_geometry, new THREE.MeshBasicMaterial({
                                    color: this.three.edge_color,
                                }))
                                const [x, y, z] = this.position(t, i, j)
                                mesh.position.set(x, y, z)
                                const [dx, dy, dz] = this.no_bias_position(t - pt, i - pi, j - pj)
                                const distance = Math.sqrt(dx*dx + dy*dy + dz*dz)
                                mesh.scale.set(1, distance / 2, 1)  // only plot half of the distance
                                // rotate
                                let axis = new THREE.Vector3( 1, 0, 0 )
                                let angle = 0
                                if (pi == i && pj == j) {
                                    if (pt < t) angle = Math.PI
                                } else {
                                    const normalize = 1 / Math.sqrt(dz*dz + dx*dx)
                                    axis = new THREE.Vector3( dz * normalize, 0, -dx * normalize )
                                    angle = -Math.atan2(Math.sqrt(dx*dx + dz*dz), dy)
                                    if (angle < 0) angle = Math.PI - angle
                                }
                                mesh.rotateOnAxis(axis, angle)
                                return mesh
                            }.bind(this)
                            if (node.edges) {
                                for (let edge of node.edges) {
                                    edge.mesh = generate_half_edge_mesh(t, i, j, edge.t, edge.i, edge.j)
                                    if (node.q_type == this.constants.QTYPE.X && !this.show_X_edges) edge.mesh.visible = false
                                    if (node.q_type == this.constants.QTYPE.Z && !this.show_Z_edges) edge.mesh.visible = false
                                    this.three.scene.add(edge.mesh)
                                }
                            }
                            if (node.boundary && node.boundary.p != undefined) {
                                node.boundary.mesh = generate_half_edge_mesh(t, i, j, node.boundary.t, node.boundary.i, node.boundary.j)
                                if (node.q_type == this.constants.QTYPE.X && !this.show_X_edges) node.boundary.mesh.visible = false
                                if (node.q_type == this.constants.QTYPE.Z && !this.show_Z_edges) node.boundary.mesh.visible = false
                                this.three.scene.add(node.boundary.mesh)
                            }
                        }
                    }
                }
            }
        },
        build_graph_given_error_rate() {  // requirement: node.px and node.pz exists
            function node_add_connection(node1, node2, p, _iterate=true) {  // DO NOT set _iterate
                if (node1.edges == undefined) node1.edges = []
                // first find node2 in its edges
                let found = false
                for (let i=0; i<node1.edges.length; ++i) {
                    let edge = node1.edges[i]
                    if (edge.t == node2.t && edge.i == node2.i && edge.j == node2.j) {
                        found = true
                        edge.p = edge.p * (1 - p) + p * (1 - edge.p)  // XOR
                        break
                    }
                }
                // create node2 edge if not found
                if (found == false) {
                    node1.edges.push({ t:node2.t, i:node2.i, j:node2.j, p })
                }
                if (_iterate) node_add_connection(node2, node1, p, false)  // add node1 to node2 connection
            }
            for (let t=0; t < this.snapshot.length-1; ++t) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        for (let e=0; e < 2; ++e) {
                            if (!this.snapshot[t][i][j]) continue
                            this.clear_errors()
                            this.snapshot[t][i][j].error = e == 0 ? this.constants.ETYPE.X : this.constants.ETYPE.Z
                            const p = e == 0 ? this.snapshot[t][i][j].px : this.snapshot[t][i][j].pz
                            this.compute_propagated_error(false)
                            const error_syndrome = this.get_error_syndrome_propagated()
                            if (error_syndrome.length == 1) {  // connect to boundary
                                const [et, ei, ej] = error_syndrome[0]
                                const boundary = this.snapshot[et][ei][ej].boundary
                                console.assert(boundary, `there must be boundary on [${et}][${ei}][${ej}]`)
                                if (boundary.p == undefined) boundary.p = 0
                                boundary.p = boundary.p * (1 - p) + p * (1 - boundary.p)
                            } else if (error_syndrome.length == 2) {  // connect to other nodes
                                const node1 = this.snapshot[error_syndrome[0][0]][error_syndrome[0][1]][error_syndrome[0][2]]
                                const node2 = this.snapshot[error_syndrome[1][0]][error_syndrome[1][1]][error_syndrome[1][2]]
                                node_add_connection(node1, node2, p)
                            }
                        }
                    }
                }
            }
            this.clear_errors()
        },
        clear_errors() {
            for (let t=0; t < this.snapshot.length; ++t) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        let node = this.snapshot[t][i][j]
                        if (!node) continue
                        node.error = this.constants.ETYPE.I
                    }
                }
            }
        },
        get_error_syndrome_propagated() {
            let error_syndrome_propagated = []
            for (let t=6; t < this.snapshot.length; t += 6) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        let node = this.snapshot[t][i][j]
                        if (!node) continue
                        if (node.n_type == this.constants.NTYPE.MEASUREMENT) {
                            if (node.q_type == this.constants.QTYPE.Z) {
                                let this_result = node.propagated == this.constants.ETYPE.I || node.propagated == this.constants.ETYPE.Z
                                const last_node = this.snapshot[t-6][i][j]
                                let last_result = last_node.propagated == this.constants.ETYPE.I || last_node.propagated == this.constants.ETYPE.Z
                                if (this_result != last_result) {
                                    error_syndrome_propagated.push([t,i,j])
                                }
                            } else {
                                let this_result = node.propagated == this.constants.ETYPE.I || node.propagated == this.constants.ETYPE.X
                                const last_node = this.snapshot[t-6][i][j]
                                let last_result = last_node.propagated == this.constants.ETYPE.I || last_node.propagated == this.constants.ETYPE.X
                                if (this_result != last_result) {
                                    error_syndrome_propagated.push([t,i,j])
                                }
                            }
                        }
                    }
                }
            }
            return error_syndrome_propagated
        },
        count_error_syndrome_propagated() {
            return this.get_error_syndrome_propagated().length
        },
        async verify_idea_all_single_error_only_has_at_most_two_syndrome() {
            for (let t=0; t < this.snapshot.length-1; ++t) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        for (let e=0; e < 2; ++e) {
                            if (!this.snapshot[t][i][j]) continue
                            this.clear_errors()
                            this.snapshot[t][i][j].error = e == 0 ? this.constants.ETYPE.X : this.constants.ETYPE.Z
                            this.compute_propagated_error()
                            // await this.sleep_ms(100)  // for visualization purpose
                            const count_error_syndrome = this.count_error_syndrome_propagated()
                            if (count_error_syndrome > 2) {
                                console.log("find error syndrome count = " + count_error_syndrome)
                                console.log(`error at [${t}][${i}][${j}]`)
                                return
                            }
                        }
                    }
                }
            }
            this.clear_errors()
            this.compute_propagated_error()
            console.log("verified: all single error only has at most two syndrome")
        },
        async verify_idea_at_most_12_neighbour_in_graph() {
            this.iterate_snapshot((node, t, i, j) => {
                if (node.edges) {
                    console.assert(node.edges.length <= 12, `find [${t}][${i}][${j}] has ${node.edges.length} edges, greater than 12`)
                }
            })
            console.log("verified: at most 12 neighbour in graph")
        },
        iterate_snapshot(func) {
            for (let t=0; t < this.snapshot.length; ++t) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        if (!this.snapshot[t][i][j]) continue
                        func(this.snapshot[t][i][j], t, i, j)
                    }
                }
            }
        }
	},
	watch: {
        show_data_qubit(show) {
            this.iterate_snapshot((node, t, i, j) => {
                if (node.n_type == this.constants.NTYPE.NONE_WITH_DATA_QUBIT) {
                    node.mesh.visible = show
                }
            })
        },
        show_X_ancilla(show) {
            this.iterate_snapshot((node, t, i, j) => {
                if (node.n_type == this.constants.NTYPE.MEASUREMENT && node.q_type == this.constants.QTYPE.X) {
                    node.mesh.visible = show
                }
            })
        },
        show_Z_ancilla(show) {
            this.iterate_snapshot((node, t, i, j) => {
                if (node.n_type == this.constants.NTYPE.MEASUREMENT && node.q_type == this.constants.QTYPE.Z) {
                    node.mesh.visible = show
                }
            })
        },
        show_vertical_line(show) {
            this.iterate_snapshot((node, t, i, j) => {
                if (node.vertical) node.vertical.visible = show
            })
        },
        show_initialization(show) {
            this.iterate_snapshot((node, t, i, j) => {
                if (node.n_type == this.constants.NTYPE.INITIALIZATION) node.mesh.visible = show
            })
        },
        show_CX_gates(show) {
            this.iterate_snapshot((node, t, i, j) => {
                if (node.n_type == this.constants.NTYPE.TARGET) {
                    for (let mesh of node.mesh) mesh.visible = show
                }
                if (node.n_type == this.constants.NTYPE.CONTROL) node.mesh.visible = show
            })
        },
        show_X_edges(show) {
            this.iterate_snapshot((node, t, i, j) => {
                if (node.q_type == this.constants.QTYPE.X) {
                    if (node.edges) for (let edge of node.edges) edge.mesh.visible = show
                    if (node.boundary && node.boundary.mesh) node.boundary.mesh.visible = show
                }
            })
        },
        show_Z_edges(show) {
            this.iterate_snapshot((node, t, i, j) => {
                if (node.q_type == this.constants.QTYPE.Z) {
                    if (node.edges) for (let edge of node.edges) edge.mesh.visible = show
                    if (node.boundary && node.boundary.mesh) node.boundary.mesh.visible = show
                }
            })
        },
	},
}
</script>

<style scoped>

.main {
	background: red;
}

</style>
