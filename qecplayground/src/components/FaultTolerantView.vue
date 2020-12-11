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
			default: 3,
		},
		T: {
			type: Number,
			default: 3,
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
			internals: { bias: { x:0, y:0, z:0 } },  // internal data

            // controllable parameters for visualization
            snapshot: null,  // [t][i][j]
            constants: null, // { QTYPE (qubit type), NTYPE (node type), etc. }
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

		// start rendering
		this.animate()

	},
	methods: {
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
            this.establish_snapshot()
        },
        build_standard_planar_code_snapshot() {
            // console.assert(this.L % 2 == 1, "L should be even")
            console.assert(this.T >= 1, "T should be at least 1, 1 is for perfect measurement condition")
            const width = 2 * this.L - 1
            const height = this.T * 6 + 1
            const always = true
            let snapshot = []
            for (let t=0; t<height; ++t) {
                let snapshot_row_0 = []
                for (let i=0; i<width; ++i) {
                    let snapshot_row_1 = []
                    for (let j=0; j<width; ++j) {
                        if (always) {  // if here exists a qubit (either data qubit or ancilla qubit)
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
                                        if (i+1 < width) {
                                            if (j % 2 == 0) n_type = this.constants.NTYPE.TARGET
                                            else n_type = this.constants.NTYPE.CONTROL
                                            connection = { i:i+1, j, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    } else {
                                        if (i-1 >= 0) {
                                            if (j % 2 == 0) n_type = this.constants.NTYPE.CONTROL
                                            else n_type = this.constants.NTYPE.TARGET
                                            connection = { i:i-1, j, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    }
                                    break
                                case 2:
                                    if (is_data_qubit) {
                                        if (j+1 < width) {
                                            if (i % 2 == 0) n_type = this.constants.NTYPE.CONTROL
                                            else n_type = this.constants.NTYPE.TARGET
                                            connection = { i, j:j+1, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    } else {
                                        if (j-1 >= 0) {
                                            if (i % 2 == 0) n_type = this.constants.NTYPE.TARGET
                                            else n_type = this.constants.NTYPE.CONTROL
                                            connection = { i, j:j-1, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    }
                                    break
                                case 3:
                                    if (is_data_qubit) {
                                        if (j-1 >= 0) {
                                            if (i % 2 == 0) n_type = this.constants.NTYPE.CONTROL
                                            else n_type = this.constants.NTYPE.TARGET
                                            connection = { i, j:j-1, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    } else {
                                        if (j+1 < width) {
                                            if (i % 2 == 0) n_type = this.constants.NTYPE.TARGET
                                            else n_type = this.constants.NTYPE.CONTROL
                                            connection = { i, j:j+1, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    }
                                    break
                                case 4:
                                    if (is_data_qubit) {
                                        if (i-1 >= 0) {
                                            if (j % 2 == 0) n_type = this.constants.NTYPE.TARGET
                                            else n_type = this.constants.NTYPE.CONTROL
                                            connection = { i:i-1, j, t }
                                        } else n_type = this.constants.NTYPE.NONE  // boundary
                                    } else {
                                        if (i+1 < width) {
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
        build_rotated_planar_code() {
            // TODO
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
        compute_propagated_error() {
            // careful: t=0 will remain propagated error, others will be recomputed
            for (let t=1; t < this.snapshot.length; ++t) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        this.snapshot[t][i][j].propagated = this.constants.ETYPE.I
                    }
                }
            }
            for (let t=0; t < this.snapshot.length-1; ++t) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        const node = this.snapshot[t][i][j]
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
            for (let t=1; t < this.snapshot.length; ++t) {
                for (let i=0; i < this.snapshot[t].length; ++i) {
                    for (let j=0; j < this.snapshot[t][i].length; ++j) {
                        const node = this.snapshot[t][i][j]
                        if (node.n_type == this.constants.NTYPE.MEASUREMENT) {
                            if (node.q_type == this.constants.QTYPE.Z) {
                                if (node.propagated == this.constants.ETYPE.X || node.propagated == this.constants.ETYPE.Y) {
                                    node.mesh.material.color = this.three.measurement_node_color_error
                                } else node.mesh.material.color = this.three.initialization_node_color_Z
                            } else {
                                if (node.propagated == this.constants.ETYPE.Z || node.propagated == this.constants.ETYPE.Y) {
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
        },
        position_middle_set_bias() {
            const [x, y, z] = this.position(0,0,0)
            let mins = [x, y, z]
            let maxs = [x, y, z]
            let position = this.position
            let search = [[this.snapshot.length-1,0,0], [0,this.snapshot[0].length-1,0], [0,0,this.snapshot[0][0].length-1]]
            search.forEach(val => {
                let pos = this.position(val[0], val[1], val[2])
                for (let i=0; i<3; ++i) {
                    if (pos[i] < mins[i]) mins[i] = pos[i]
                    if (pos[i] > maxs[i]) maxs[i] = pos[i]
                }
            })
            this.internals.bias.x = -0.5 * (maxs[0] - mins[0])
            this.internals.bias.y = -0.5 * (maxs[1] - mins[1])
            this.internals.bias.z = -0.5 * (maxs[2] - mins[2])
        },
        position(t, i, j) {
            const x = j + this.internals.bias.x
            const y = t * this.constants.VERTICAL_INTERVAL + this.internals.bias.y
            const z = i + this.internals.bias.z
            return [x, y, z]
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
            const control_tube = 0.02
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
            this.three.CX_control_geometry = new THREE.SphereBufferGeometry( 0.05, 12, 6 )
            this.three.CX_control_color = new THREE.Color( 'black' )
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
                        }
                    }
                }
            }
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
