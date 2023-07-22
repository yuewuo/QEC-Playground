// 3d related apis

import * as THREE from 'three'
import { OrbitControls } from './node_modules/three/examples/jsm/controls/OrbitControls.js'
import { ConvexGeometry } from './node_modules/three/examples/jsm/geometries/ConvexGeometry.js'
import Stats from './node_modules/three/examples/jsm/libs/stats.module.js'
import GUI from './node_modules/three/examples/jsm/libs/lil-gui.module.min.js'


if (typeof window === 'undefined' || typeof document === 'undefined') {
    global.THREE = THREE
    global.mocker = await import('./mocker.js')
}

// to work both in browser and nodejs
if (typeof Vue === 'undefined') {
    global.Vue = await import('vue')
}
const { ref, shallowRef, reactive, watch, computed } = Vue

const urlParams = new URLSearchParams(window.location.search)
export const root = document.documentElement

export const is_mock = typeof mockgl !== 'undefined'
export const webgl_renderer_context = is_mock ? mockgl : () => undefined

export const window_inner_width = ref(0)
export const window_inner_height = ref(0)
function on_resize() {
    window_inner_width.value = window.innerWidth
    window_inner_height.value = window.innerHeight
}
on_resize()
window.addEventListener('resize', on_resize)
window.addEventListener('orientationchange', on_resize)

export const sizes = reactive({
    control_bar_width: 0,
    canvas_width: 0,
    canvas_height: 0,
    scale: 1,
})

watch([window_inner_width, window_inner_height], () => {
    sizes.scale = window_inner_width.value / 1920
    if (sizes.scale > window_inner_height.value / 1080) {  // ultra-wide
        sizes.scale = window_inner_height.value / 1080
    }
    if (sizes.scale < 0.5) {
        sizes.scale = 0.5
    }
    if (window_inner_width.value * 0.9 < 300) {
        sizes.scale = window_inner_width.value / 600 * 0.9
    }
    root.style.setProperty('--s', sizes.scale)
    // sizes.scale = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--s'))
    sizes.control_bar_width = 600 * sizes.scale
    sizes.canvas_width = window_inner_width.value - sizes.control_bar_width
    sizes.canvas_height = window_inner_height.value
}, { immediate: true })
if (is_mock) {
    sizes.canvas_width = mocker.mock_canvas_width
    sizes.canvas_height = mocker.mock_canvas_height
}

export const scene = new THREE.Scene()
scene.background = new THREE.Color(0xffffff)  // for better image output
scene.add(new THREE.AmbientLight(0xffffff))
window.scene = scene
export const perspective_camera = new THREE.PerspectiveCamera(75, sizes.canvas_width / sizes.canvas_height, 0.1, 10000)
const orthogonal_camera_init_scale = 6
export const orthogonal_camera = new THREE.OrthographicCamera(sizes.canvas_width / sizes.canvas_height * (-orthogonal_camera_init_scale)
    , sizes.canvas_width / sizes.canvas_height * orthogonal_camera_init_scale, orthogonal_camera_init_scale, -orthogonal_camera_init_scale, 0.1, 100000)
export const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, context: webgl_renderer_context() })

document.body.appendChild(renderer.domElement)

watch(sizes, () => {
    perspective_camera.aspect = sizes.canvas_width / sizes.canvas_height
    perspective_camera.updateProjectionMatrix()
    orthogonal_camera.left = sizes.canvas_width / sizes.canvas_height * (-orthogonal_camera_init_scale)
    orthogonal_camera.right = sizes.canvas_width / sizes.canvas_height * (orthogonal_camera_init_scale)
    orthogonal_camera.updateProjectionMatrix()
    renderer.setSize(sizes.canvas_width, sizes.canvas_height, false)
    const ratio = window.devicePixelRatio  // looks better on devices with a high pixel ratio, such as iPhones with Retina displays
    renderer.setPixelRatio(ratio)
    const canvas = renderer.domElement
    canvas.width = sizes.canvas_width * ratio
    canvas.height = sizes.canvas_height * ratio
    canvas.style.width = `${sizes.canvas_width}px`
    canvas.style.height = `${sizes.canvas_height}px`
}, { immediate: true })

export const orbit_control_perspective = new OrbitControls(perspective_camera, renderer.domElement)
export const orbit_control_orthogonal = new OrbitControls(orthogonal_camera, renderer.domElement)
export const enable_control = ref(true)
watch(enable_control, (enabled) => {
    orbit_control_perspective.enabled = enabled
    orbit_control_orthogonal.enabled = enabled
}, { immediate: true })
window.enable_control = enable_control

export const use_perspective_camera = ref(false)
export const camera = computed(() => {
    return use_perspective_camera.value ? perspective_camera : orthogonal_camera
})
window.camera = camera
export const orbit_control = computed(() => {
    return use_perspective_camera.value ? orbit_control_perspective : orbit_control_orthogonal
})

export function reset_camera_position(direction = "top") {
    for (let [camera, control, distance] of [[perspective_camera, orbit_control_perspective, 8], [orthogonal_camera, orbit_control_orthogonal, 1000]]) {
        control.reset()
        camera.position.x = (direction == "left" ? -distance : 0)
        camera.position.y = (direction == "top" ? distance : 0)
        camera.position.z = (direction == "front" ? distance : 0)
        camera.lookAt(0, 0, 0)
    }
}
reset_camera_position()

// const axesHelper = new THREE.AxesHelper( 5 )
// scene.add( axesHelper )

var stats
export const show_stats = ref(false)
if (!is_mock) {
    stats = Stats()
    document.body.appendChild(stats.dom)
    watch(show_stats, function () {
        if (show_stats.value) {
            stats.dom.style.display = "block"
        } else {
            stats.dom.style.display = "none"
        }
    }, { immediate: true })
    watch(sizes, () => {
        stats.dom.style.transform = `scale(${sizes.scale})`
        stats.dom.style["transform-origin"] = "left top"
    }, { immediate: true })
}

export function animate() {
    requestAnimationFrame(animate)
    orbit_control.value.update()
    renderer.render(scene, camera.value)
    if (stats) stats.update()
}

// commonly used vectors
const zero_vector = new THREE.Vector3(0, 0, 0)
const unit_up_vector = new THREE.Vector3(0, 1, 0)
export const t_scale = parseFloat(urlParams.get('t_scale') || 1 / 3)

// currently displaying elements within a range
export const t_length = ref(1)
export const t_range = ref({ min: 0, max: 0 })
export function in_t_range(t) {
    return t >= t_range.value.min && t < t_range.value.max
}
export function in_t_range_any(t1, t2) {
    const ts = Math.min(t1, t2)
    const te = Math.max(t1, t2)
    return !(ts >= t_range.value.max || te < t_range.value.min)
}

// create common geometries
const segment = parseInt(urlParams.get('segment') || 128)  // higher segment will consume more GPU resources
const qubit_radius = parseFloat(urlParams.get('qubit_radius') || 0.15)
export const qubit_radius_scale = ref(1)
const scaled_qubit_radius = computed(() => {
    return qubit_radius * qubit_radius_scale.value
})
const qubit_geometry = new THREE.SphereGeometry(qubit_radius, segment, segment)
const idle_gate_radius = parseFloat(urlParams.get('idle_gate_radius') || 0.025)
const idle_gate_radius_scale = ref(1)
const idle_gate_geometry = new THREE.CylinderGeometry(idle_gate_radius, idle_gate_radius, 1, segment, 1, true)
idle_gate_geometry.translate(0, 0.5, 0)
const initialization_geometry = new THREE.ConeBufferGeometry(0.1, 0.15, 32)
const control_qubit_geometry = new THREE.SphereBufferGeometry(0.05, 12, 6)
const control_line_radius = 0.02
const control_line_geometry = new THREE.CylinderGeometry(control_line_radius, control_line_radius, 1, segment, 1, true)
control_line_geometry.translate(0, 0.5, 0)
const CX_target_radius = 0.15
const CX_target_geometries = [
    new THREE.TorusBufferGeometry(CX_target_radius, control_line_radius, 16, 32),
    new THREE.CylinderBufferGeometry(control_line_radius, control_line_radius, 2 * CX_target_radius, 6),
    new THREE.CylinderBufferGeometry(control_line_radius, control_line_radius, 2 * CX_target_radius, 6),
]
CX_target_geometries[0].rotateX(Math.PI / 2)
CX_target_geometries[1].rotateX(Math.PI / 2)
CX_target_geometries[2].rotateZ(Math.PI / 2)
const CY_target_radius = 0.2
const CY_target_Y_length = 0.12
const CY_target_geometries = [
    new THREE.TorusBufferGeometry(CY_target_radius, control_line_radius, 16, 4),
    new THREE.CylinderBufferGeometry(control_line_radius, control_line_radius, CY_target_Y_length, 6),
    new THREE.CylinderBufferGeometry(control_line_radius, control_line_radius, CY_target_Y_length, 6),
    new THREE.CylinderBufferGeometry(control_line_radius, control_line_radius, CY_target_Y_length, 6),
]
CY_target_geometries[0].rotateX(Math.PI / 2)
CY_target_geometries[0].rotateY(Math.PI / 4)
CY_target_geometries[1].translate(0, CY_target_Y_length / 2, 0)
CY_target_geometries[2].translate(0, CY_target_Y_length / 2, 0)
CY_target_geometries[3].translate(0, CY_target_Y_length / 2, 0)
CY_target_geometries[1].rotateX(Math.PI / 2)
CY_target_geometries[1].rotateY(- 5 * Math.PI / 6)
CY_target_geometries[2].rotateX(Math.PI / 2)
CY_target_geometries[2].rotateY(5 * Math.PI / 6)
CY_target_geometries[3].rotateX(Math.PI / 2)
const model_graph_edge_radius = parseFloat(urlParams.get('model_graph_edge_radius') || 0.03)
const model_graph_edge_scale = ref(1)
const model_graph_edge_geometry = new THREE.CylinderGeometry(model_graph_edge_radius, model_graph_edge_radius, 1, segment, 1, true)
model_graph_edge_geometry.translate(0, 0.5, 0)
const model_graph_vertex_radius = parseFloat(urlParams.get('model_graph_vertex_radius') || 0.12)
const model_graph_vertex_scale = ref(1)
const model_graph_vertex_geometry = new THREE.SphereGeometry(model_graph_vertex_radius, segment, segment)
const noise_model_pauli_geometry = new THREE.CapsuleGeometry(0.05, 0.04, 8, 16)
noise_model_pauli_geometry.translate(0, t_scale * 0.3, 0)
const noise_model_erasure_geometry = new THREE.CapsuleGeometry(0.05, 0.04, 8, 16)
noise_model_erasure_geometry.translate(0, t_scale * 0.65, 0)
const singular_hyperedge_geometry = new THREE.CylinderGeometry(model_graph_vertex_radius * 2, model_graph_vertex_radius * 2, 0.01, segment, 1, false)
// singular_hyperedge_geometry.translate(0, -model_graph_vertex_radius, 0)
const normal_hyperedge_geometry = new THREE.CylinderGeometry(model_graph_edge_radius, model_graph_edge_radius, 1, segment, 1, true)
normal_hyperedge_geometry.translate(0, 0.5, 0)
const tri_hyperedge_geometry = new THREE.CylinderGeometry(model_graph_edge_radius * 1.5, model_graph_edge_radius * 1.5, 1, segment, 1, true)
tri_hyperedge_geometry.translate(0, 0.5, 0)
const quad_hyperedge_geometry = new THREE.CylinderGeometry(model_graph_edge_radius * 2, model_graph_edge_radius * 2, 1, segment, 1, true)
quad_hyperedge_geometry.translate(0, 0.5, 0)
const hyperedge_geometries = [
    singular_hyperedge_geometry,
    normal_hyperedge_geometry,
    tri_hyperedge_geometry,
    quad_hyperedge_geometry,
]
function get_hyperedge_geometry(hyperedge_degree) {
    if (hyperedge_degree - 1 < hyperedge_geometries.length) return hyperedge_geometries[hyperedge_degree - 1]
    return hyperedge_geometries[hyperedge_geometries.length - 1]
}
const matching_edge_radius = parseFloat(urlParams.get('matching_edge_radius') || 0.06)
const matching_edge_geometry = new THREE.CylinderGeometry(matching_edge_radius, matching_edge_radius, 1, segment, 1, true)
matching_edge_geometry.translate(0, 0.5, 0)

// measurement bits
const measurement_radius = parseFloat(urlParams.get('measurement_radius') || 0.06)
export const measurement_radius_scale = ref(1)
const measurement_geometry = new THREE.SphereGeometry(measurement_radius, segment, segment)
const defect_measurement_radius = parseFloat(urlParams.get('defect_measurement_radius') || 0.15)
export const defect_measurement_radius_scale = ref(1)
const defect_measurement_geometry = new THREE.SphereGeometry(defect_measurement_radius, segment, segment)

// error pattern geometries
const error_line_radius = 0.04
const error_X_radius = 0.3
const error_X_geometries = [
    new THREE.TorusBufferGeometry(error_X_radius, error_line_radius, 16, 32),
    new THREE.CylinderBufferGeometry(error_line_radius, error_line_radius, 2 * error_X_radius, 6),
    new THREE.CylinderBufferGeometry(error_line_radius, error_line_radius, 2 * error_X_radius, 6),
]
error_X_geometries[0].rotateX(Math.PI / 2)
error_X_geometries[1].rotateX(Math.PI / 2)
error_X_geometries[1].rotateY(Math.PI / 4)
error_X_geometries[2].rotateZ(Math.PI / 2)
error_X_geometries[2].rotateY(Math.PI / 4)
const error_Y_radius = 0.4
const error_Y_length = 0.24
const error_Y_geometries = [
    new THREE.TorusBufferGeometry(error_Y_radius, error_line_radius, 16, 4),
    new THREE.CylinderBufferGeometry(error_line_radius, error_line_radius, error_Y_length, 6),
    new THREE.CylinderBufferGeometry(error_line_radius, error_line_radius, error_Y_length, 6),
    new THREE.CylinderBufferGeometry(error_line_radius, error_line_radius, error_Y_length, 6),
]
error_Y_geometries[0].rotateX(Math.PI / 2)
error_Y_geometries[0].rotateY(Math.PI / 4)
error_Y_geometries[1].translate(0, error_Y_length / 2, 0)
error_Y_geometries[2].translate(0, error_Y_length / 2, 0)
error_Y_geometries[3].translate(0, error_Y_length / 2, 0)
error_Y_geometries[1].rotateX(Math.PI / 2)
error_Y_geometries[1].rotateY(- 5 * Math.PI / 6)
error_Y_geometries[2].rotateX(Math.PI / 2)
error_Y_geometries[2].rotateY(5 * Math.PI / 6)
error_Y_geometries[3].rotateX(Math.PI / 2)
const error_Z_radius = 0.4
const error_Z_length = 0.3
const error_Z_geometries = [
    new THREE.TorusBufferGeometry(error_Z_radius, error_line_radius, 16, 4),
    new THREE.CylinderBufferGeometry(error_line_radius, error_line_radius, error_Z_length, 6),
    new THREE.CylinderBufferGeometry(error_line_radius, error_line_radius, error_Z_length, 6),
    new THREE.CylinderBufferGeometry(error_line_radius, error_line_radius, error_Z_length * Math.sqrt(61) / 5, 6),
    // new THREE.CylinderBufferGeometry( error_line_radius, error_line_radius, error_Z_length, 6 ),
    // new THREE.CylinderBufferGeometry( error_line_radius, error_line_radius, error_Z_length, 6 ),
]
error_Z_geometries[0].rotateX(Math.PI / 2)
error_Z_geometries[0].rotateY(Math.PI / 4)
// error_Z_geometries[1].rotateX(Math.PI / 2)
error_Z_geometries[1].rotateZ(Math.PI / 2)
error_Z_geometries[1].translate(0, 0, error_Z_length * 0.6)
error_Z_geometries[2].rotateZ(Math.PI / 2)
error_Z_geometries[2].translate(0, 0, -error_Z_length * 0.6)
error_Z_geometries[3].rotateZ(Math.PI / 2)
error_Z_geometries[3].rotateY(1.0)
const detected_erasure_radius = 0.5
const detected_erasure_geometry = new THREE.TorusBufferGeometry(detected_erasure_radius, error_line_radius, 16, 4)
detected_erasure_geometry.rotateX(Math.PI / 2)
detected_erasure_geometry.rotateY(Math.PI / 4)

export const sequential_colors = [  // https://quasar.dev/style/color-palette
    ["blue", 0x2196f3],
    ["green", 0x4caf50],
    ["deep-purple", 0x673ab7],
    ["brown", 0x795548],
    ["lime", 0xcddc39],
    ["pink", 0xe91e63],
    ["purple", 0x9c27b0],
    ["deep-purple", 0x673ab7],
    ["indigo", 0x3f51b5],
    ["light-blue", 0x03a9f4],
    ["cyan", 0x00bcd4],
    ["teal", 0x009688],
    ["green", 0x4caf50],
    ["light-green", 0x8bc34a],
    ["lime", 0xcddc39],
    ["yellow", 0xffeb3b],
    ["amber", 0xffc107],
    ["orange", 0xff9800],
    ["deep-orange", 0xff5722],
    ["grey", 0x9e9e9e],
    ["blue-grey", 0x607d8b],
    ["red", 0xf44336],
]
export const mapping_colors = {}
for (const [name, color] of sequential_colors) {
    mapping_colors[name] = color
}

// create common materials
function build_solid_material(color) {
    return new THREE.MeshStandardMaterial({
        color: color,
        opacity: 1,
        transparent: true,
        side: THREE.FrontSide,
    })
}
export const const_color = {
    "X": 0x00CC00,
    "Z": 0x00C0FF,
    "Y": 0xF5B042,
}
export const qubit_materials = {
    "Data": build_solid_material(0xAAAAAA),
    "StabX": build_solid_material(const_color.X),
    "StabZ": build_solid_material(const_color.Z),
    "StabXZZXLogicalX": build_solid_material(0xF4CCCC),
    "StabXZZXLogicalZ": build_solid_material(0xF4CCCC),
    "StabY": build_solid_material(const_color.Y),
    "Unknown": build_solid_material(0xFF0000),
}
export function get_qubit_material(qubit_type) {
    const qubit_material = qubit_materials[qubit_type]
    if (qubit_material == null) {
        console.error(`unknown qubit_type: ${qubit_type}`)
        return qubit_materials["Unknown"]
    }
    return qubit_material
}
export const gate_materials = {
    "InitializeZ": build_solid_material(const_color.Z),
    "InitializeX": build_solid_material(const_color.X),
    "CXGateControl": build_solid_material(0x000000),
    "CXGateTarget": build_solid_material(0x000000),
    "CYGateControl": build_solid_material(0x000000),
    "CYGateTarget": build_solid_material(0x000000),
    "CZGate": build_solid_material(0x000000),
    "MeasureZ": build_solid_material(const_color.Z),
    "MeasureX": build_solid_material(const_color.X),
    "Unknown": build_solid_material(0xFF0000),
}
export function get_gate_material(gate_type) {
    const gate_material = gate_materials[gate_type]
    if (gate_material == null) {
        console.error(`unknown qubit_type: ${qubit_type}`)
        return gate_materials["Unknown"]
    }
    return gate_material
}
export const error_materials = {
    "X": build_solid_material(const_color.X),
    "Y": build_solid_material(const_color.Y),
    "Z": build_solid_material(const_color.Z),
}
export const detected_erasure_material = build_solid_material("purple")
export const model_graph_edge_material = build_solid_material(0x000000)
export const model_graph_vertex_material_vec = []
for (const [name, color] of sequential_colors) {
    model_graph_vertex_material_vec.push(build_solid_material(color))
}
export const noise_model_pauli_material = build_solid_material(0xFF0000)
export const noise_model_erasure_material = build_solid_material("purple")
export const model_hypergraph_edge_material = new THREE.MeshStandardMaterial({
    color: 0x000000,
    opacity: 0.2,
    transparent: true,
    side: THREE.FrontSide,
})
export const tailored_model_graph_vertex_material = build_solid_material("black")
export const tailored_model_graph_virtual_vertex_material = build_solid_material(0xff9800)
export const tailored_model_graph_corner_vertex_material = build_solid_material(0xf44336)
export const tailored_model_graph_edge_material_vec = []
for (let i = 0; i < 3; ++i) {
    tailored_model_graph_edge_material_vec.push(build_solid_material(sequential_colors[i + 1][1]))
}

build_solid_material(0x006699)
export const idle_gate_material = new THREE.MeshStandardMaterial({
    color: 0x000000,
    opacity: 0.1,
    transparent: true,
    side: THREE.FrontSide,
})
export const measurement_material = new THREE.MeshStandardMaterial({
    color: 0x000055,
    opacity: 1,
    transparent: true,
    side: THREE.FrontSide,
})
export const virtual_measurement_material = new THREE.MeshStandardMaterial({
    color: 0xFFFF00,
    opacity: 0.5,
    transparent: true,
    side: THREE.FrontSide,
})
export const defect_measurement_material = new THREE.MeshStandardMaterial({
    color: 0xFF0000,
    opacity: 1,
    transparent: true,
    side: THREE.FrontSide,
})
function build_outline_material(color) {
    return new THREE.MeshStandardMaterial({
        color: color,
        opacity: 1,
        transparent: true,
        side: THREE.BackSide,
    })
}
export const qubit_outline_material = build_outline_material(0x000000)
export const measurement_outline_material = build_outline_material(0x000000)
export const virtual_measurement_outline_material = build_outline_material(0x000000)
export const defect_measurement_outline_material = build_outline_material(0x000000)
export const hover_material = new THREE.MeshStandardMaterial({  // when mouse is on this object (vertex or edge)
    color: 0x6FDFDF,
    side: THREE.DoubleSide,
})
export const selected_material = new THREE.MeshStandardMaterial({  // when mouse is on this object (vertex or edge)
    color: 0x4B7BE5,
    side: THREE.DoubleSide,
})

// meshes that can be reused across different cases
export var qubit_meshes = []
export const outline_ratio = ref(1.2)
export var qubit_outline_meshes = []
export var measurement_outline_meshes = []
export var idle_gate_meshes = []
export var gate_vec_meshes = []
export var model_graph_edge_vec_meshes = []
export var model_graph_vertex_meshes = []
export var noise_model_pauli_meshes = []
export var noise_model_erasure_meshes = []
export var model_hypergraph_vertex_meshes = []
export var model_hypergraph_edge_vec_meshes = []
export var tailored_model_graph_vertex_meshes = []
export var tailored_model_graph_edge_vec_meshes = []

// meshes of a specific case
export var defect_measurement_meshes = []
export var defect_measurement_outline_meshes = []
export var error_pattern_vec_meshes = []
export var correction_vec_meshes = []
export var contributed_noise_sources = []
export var detected_erasure_meshes = []
export var matching_meshes = []

// update the sizes of objects
watch(qubit_radius_scale, (newVal, oldVal) => {
    qubit_geometry.scale(1 / oldVal, 1 / oldVal, 1 / oldVal)
    qubit_geometry.scale(newVal, newVal, newVal)
})
watch(idle_gate_radius_scale, (newVal, oldVal) => {
    idle_gate_geometry.scale(1 / oldVal, 1, 1 / oldVal)
    idle_gate_geometry.scale(newVal, 1, newVal)
})
watch(model_graph_edge_scale, (newVal, oldVal) => {
    model_graph_edge_geometry.scale(1 / oldVal, 1, 1 / oldVal)
    model_graph_edge_geometry.scale(newVal, 1, newVal)
})
watch(model_graph_vertex_scale, (newVal, oldVal) => {
    model_graph_vertex_geometry.scale(1 / oldVal, 1, 1 / oldVal)
    model_graph_vertex_geometry.scale(newVal, 1, newVal)
})
watch(defect_measurement_radius_scale, (newVal, oldVal) => {
    defect_measurement_geometry.scale(1 / oldVal, 1 / oldVal, 1 / oldVal)
    defect_measurement_geometry.scale(newVal, newVal, newVal)
})
function update_mesh_outline(mesh) {
    mesh.scale.x = outline_ratio.value
    mesh.scale.y = outline_ratio.value
    mesh.scale.z = outline_ratio.value
}
watch([outline_ratio, measurement_radius_scale], () => {
    for (let mesh of measurement_outline_meshes) {
        update_mesh_outline(mesh)
    }
})

// helper functions
export function compute_vector3(data_position) {
    let vector = new THREE.Vector3(0, 0, 0)
    load_position(vector, data_position)
    return vector
}
export function load_position(mesh_position, data_position) {
    mesh_position.z = data_position.x
    mesh_position.x = data_position.y
    mesh_position.y = data_position.t * t_scale
}

function build_2d_array(vertical, horizontal, value = (i, j) => null) {
    let array = []
    for (let i = 0; i < vertical; ++i) {
        let row = []
        for (let j = 0; j < horizontal; ++j) {
            row.push(value(i, j))
        }
        array.push(row)
    }
    return array
}

function dispose_mesh_2d_array(array) {
    for (let row of array) {
        for (let mesh of row) {
            if (mesh != null) {
                scene.remove(mesh)
            }
        }
    }
}

function build_3d_array(height, vertical, horizontal, value = (t, i, j) => null) {
    let array = []
    for (let t = 0; t < height; ++t) {
        let layer = []
        for (let i = 0; i < vertical; ++i) {
            let row = []
            for (let j = 0; j < horizontal; ++j) {
                row.push(value(t, i, j))
            }
            layer.push(row)
        }
        array.push(layer)
    }
    return array
}

function dispose_mesh_3d_array(array) {
    for (let layer of array) {
        for (let row of layer) {
            for (let mesh of row) {
                if (mesh != null) {
                    if (Array.isArray(mesh)) {
                        for (let sub_mesh of mesh) {
                            scene.remove(sub_mesh)
                        }
                    } else {
                        scene.remove(mesh)
                    }
                }
            }
        }
    }
}

function dispose_mesh_1d_array(array) {
    for (let mesh of array) {
        if (Array.isArray(mesh)) {
            for (let sub_mesh of mesh) {
                scene.remove(sub_mesh)
            }
        } else {
            scene.remove(mesh)
        }
    }
}

export function get_position(position_str) {
    const matched_pos = position_str.match(/^\[(\d+)\]\[(\d+)\]\[(\d+)\]$/)
    return {
        t: parseInt(matched_pos[1]),
        i: parseInt(matched_pos[2]),
        j: parseInt(matched_pos[3]),
    }
}

export function to_position_str(position) {
    return `[${position.t}][${position.i}][${position.j}]`
}

export function get_defect_vertices(defect_vertices_str) {
    const defect_vertices = []
    for (let position_str of defect_vertices_str.split("+")) {
        defect_vertices.push(get_position(position_str))
    }
    return defect_vertices
}

function get_url_bool(name, default_value = true) {
    let value = urlParams.get(name)
    if (value == null) {
        return default_value
    }
    return value == "true"
}

// display options
export const display_qubits = ref(get_url_bool("display_qubits", true))
export function update_visible_qubit() {
    const qecp_data = active_qecp_data.value
    for (let i = 0; i < qecp_data.simulator.vertical; ++i) {
        for (let j = 0; j < qecp_data.simulator.horizontal; ++j) {
            if (qubit_meshes[i][j]) qubit_meshes[i][j].visible = display_qubits.value
            if (qubit_outline_meshes[i][j]) {
                qubit_outline_meshes[i][j].visible = display_qubits.value
                update_mesh_outline(qubit_outline_meshes[i][j])
            }
        }
    }
}
watch([display_qubits, outline_ratio], update_visible_qubit)
export const display_idle_sticks = ref(get_url_bool("display_idle_sticks", true))
export function update_visible_idle_sticks() {
    const qecp_data = active_qecp_data.value
    for (let t = 0; t < qecp_data.simulator.height; ++t) {
        for (let i = 0; i < qecp_data.simulator.vertical; ++i) {
            for (let j = 0; j < qecp_data.simulator.horizontal; ++j) {
                if (idle_gate_meshes[t][i][j]) idle_gate_meshes[t][i][j].visible = display_idle_sticks.value && in_t_range(t)
            }
        }
    }
}
watch([display_idle_sticks, t_range], update_visible_idle_sticks, { deep: true })
// gates
export const display_gates = ref(get_url_bool("display_gates", true))
export function update_visible_gates() {
    const qecp_data = active_qecp_data.value
    for (let t = 0; t < qecp_data.simulator.height; ++t) {
        for (let i = 0; i < qecp_data.simulator.vertical; ++i) {
            for (let j = 0; j < qecp_data.simulator.horizontal; ++j) {
                if (gate_vec_meshes[t][i][j]) {
                    for (const mesh of gate_vec_meshes[t][i][j]) {
                        mesh.visible = display_gates.value && in_t_range(t)
                    }
                }
            }
        }
    }
}
watch([display_gates, t_range], update_visible_gates, { deep: true })
// measurement
export const display_measurements = ref(get_url_bool("display_measurements", true))
export function update_visible_defect_measurement() {
    console.assert(defect_measurement_meshes.length == defect_measurement_outline_meshes.length)
    for (let i = 0; i < defect_measurement_meshes.length; ++i) {
        const mesh = defect_measurement_meshes[i]
        const outline_mesh = defect_measurement_outline_meshes[i]
        const t = mesh.userData.t
        mesh.visible = display_measurements.value && in_t_range(t)
        outline_mesh.visible = display_measurements.value && in_t_range(t)
        update_mesh_outline(outline_mesh)
    }
}
watch([display_measurements, outline_ratio, t_range], update_visible_defect_measurement, { deep: true })
// error pattern
export const display_error_pattern = ref(get_url_bool("display_error_pattern", false))
export function update_visible_error_pattern() {
    const qecp_data = active_qecp_data.value
    const case_idx = active_case_idx.value
    const active_case = qecp_data.cases[case_idx]
    for (let [idx, [position_str, error]] of Object.entries(active_case.error_pattern).entries()) {
        const position = get_position(position_str)
        const error_pattern_vec_mesh = error_pattern_vec_meshes[idx]
        for (const mesh of error_pattern_vec_mesh) {
            mesh.visible = display_error_pattern.value && in_t_range(position.t)
        }
    }
    if (active_case.detected_erasures) {
        for (let [idx, position_str] of Object.entries(active_case.detected_erasures)) {
            const position = get_position(position_str)
            const mesh = detected_erasure_meshes[idx]
            mesh.visible = display_error_pattern.value && in_t_range(position.t)
        }
    }
}
watch([display_error_pattern, t_range], update_visible_error_pattern, { deep: true })
// correction
export const display_correction = ref(get_url_bool("display_correction", true))
export function update_visible_correction() {
    for (let correction_vec_mesh of correction_vec_meshes) {
        for (const mesh of correction_vec_mesh) {
            mesh.visible = display_correction.value
        }
    }
}
watch([display_correction], update_visible_correction)
// model graph
export const existed_model_graph = ref(false)
export const display_model_graph = ref(get_url_bool("display_model_graph", false))
export const model_graph_regions = ref(0)
export const model_graph_region_display = ref([])
export function update_visible_model_graph() {
    const active_regions = {}
    for (const active_region of Object.values(model_graph_region_display.value)) active_regions[active_region] = true
    for (let t = 0; t < qecp_data.simulator.height; ++t) {
        for (let i = 0; i < qecp_data.simulator.vertical; ++i) {
            for (let j = 0; j < qecp_data.simulator.horizontal; ++j) {
                const mesh = model_graph_vertex_meshes[t][i][j]
                if (mesh != null) {
                    const region_idx = mesh.userData.region_idx
                    const visible = active_regions[region_idx] == true && display_model_graph.value
                    mesh.visible = visible && in_t_range(t)
                    for (const edge_mesh of model_graph_edge_vec_meshes[t][i][j]) {
                        edge_mesh.visible = visible && in_t_range(t)
                    }
                }
            }
        }
    }
}
watch([model_graph_region_display, display_model_graph, t_range], update_visible_model_graph, { deep: true })
// model hypergraph
export const existed_model_hypergraph = ref(false)
export const display_model_hypergraph = ref(get_url_bool("display_model_hypergraph", false))
export function update_visible_model_hypergraph() {
    if (qecp_data.model_hypergraph == null) return
    // if any defect vertices is in the range, display the corresponding edge
    for (let vertex_index = 0; vertex_index < qecp_data.model_hypergraph.vertex_positions.length; ++vertex_index) {
        let mesh = model_hypergraph_vertex_meshes[vertex_index]
        const tij = get_position(qecp_data.model_hypergraph.vertex_positions[vertex_index])
        mesh.visible = display_model_hypergraph.value && in_t_range(tij.t)
    }
    for (let edge_index = 0; edge_index < qecp_data.model_hypergraph.weighted_edges.length; ++edge_index) {
        const [min_t, max_t] = qecp_data.model_hypergraph.edge_min_max_t[edge_index]
        const visible = display_model_hypergraph.value && min_t <= t_range.value.max && max_t >= t_range.value.min
        for (let mesh of model_hypergraph_edge_vec_meshes[edge_index]) {
            mesh.visible = visible
        }
    }
}
watch([display_model_hypergraph, t_range], update_visible_model_hypergraph, { deep: true })
// tailored model graph
export const existed_tailored_model_graph = ref(false)
export const display_tailored_model_graph = ref(get_url_bool("display_tailored_model_graph", false))
export const tailored_model_graph_region_display = ref([0, 1, 2])  // by default display all three graphs
export function update_visible_tailored_model_graph() {
    const active_regions = {}
    for (const active_region of Object.values(tailored_model_graph_region_display.value)) active_regions[active_region] = true
    for (let t = 0; t < qecp_data.simulator.height; ++t) {
        for (let i = 0; i < qecp_data.simulator.vertical; ++i) {
            for (let j = 0; j < qecp_data.simulator.horizontal; ++j) {
                const mesh = tailored_model_graph_vertex_meshes[t][i][j]
                if (mesh != null) {
                    mesh.visible = display_tailored_model_graph.value && in_t_range(t)
                    for (const edge_mesh of tailored_model_graph_edge_vec_meshes[t][i][j]) {
                        let tsg = edge_mesh.userData.tsg
                        const visible = display_tailored_model_graph.value && active_regions[tsg] == true
                        edge_mesh.visible = visible && in_t_range(t)
                    }
                }
            }
        }
    }
}
watch([tailored_model_graph_region_display, display_tailored_model_graph, t_range], update_visible_tailored_model_graph, { deep: true })
// noise model
export const existed_noise_model = ref(false)
export const display_noise_model_pauli = ref(get_url_bool("display_noise_model_pauli", true))
export const display_noise_model_erasure = ref(get_url_bool("display_noise_model_erasure", true))
export function update_visible_noise_model() {
    for (let t = 0; t < qecp_data.simulator.height; ++t) {
        for (let i = 0; i < qecp_data.simulator.vertical; ++i) {
            for (let j = 0; j < qecp_data.simulator.horizontal; ++j) {
                const pauli_mesh = noise_model_pauli_meshes[t][i][j]
                if (pauli_mesh != null) {
                    pauli_mesh.visible = display_noise_model_pauli.value && in_t_range(t)
                }
                const erasure_mesh = noise_model_erasure_meshes[t][i][j]
                if (erasure_mesh != null) {
                    erasure_mesh.visible = display_noise_model_erasure.value && in_t_range(t)
                }
            }
        }
    }
}
watch([display_noise_model_pauli, display_noise_model_erasure, t_range], update_visible_noise_model, { deep: true })

export async function refresh_qecp_data() {
    // console.log("refresh_qecp_data")
    if (active_qecp_data.value != null) {  // no qecp data provided
        const qecp_data = active_qecp_data.value
        const nodes = qecp_data.simulator.nodes
        // clear hover and select
        current_hover.value = null
        current_selected.value = null
        await Vue.nextTick()
        await Vue.nextTick()
        // constants
        const height = qecp_data.simulator.height
        const t_bias = -height / 2
        const vertical = qecp_data.simulator.vertical
        const horizontal = qecp_data.simulator.horizontal
        // update t range
        t_length.value = height
        t_range.value.max = parseFloat(urlParams.get('t_max') || height)
        t_range.value.min = parseFloat(urlParams.get('t_min') || 0)
        // draw qubits at t=-1
        dispose_mesh_2d_array(qubit_meshes)
        dispose_mesh_2d_array(qubit_outline_meshes)
        qubit_meshes = build_2d_array(vertical, horizontal)
        qubit_outline_meshes = build_2d_array(vertical, horizontal)
        for (let i = 0; i < vertical; ++i) {
            for (let j = 0; j < horizontal; ++j) {
                const qubit = nodes[0][i][j]
                if (qubit != null && !qubit.v) {
                    const position = qecp_data.simulator.positions[i][j]
                    const display_position = {
                        t: -1 + t_bias,
                        x: position.x,
                        y: position.y,
                    }
                    // qubit
                    const qubit_material = get_qubit_material(qubit.q)
                    const qubit_mesh = new THREE.Mesh(qubit_geometry, qubit_material)
                    qubit_mesh.userData = {
                        type: "qubit",
                        qubit_type: qubit.q,
                        i: i,
                        j: j,
                    }
                    scene.add(qubit_mesh)
                    load_position(qubit_mesh.position, display_position)
                    qubit_meshes[i][j] = qubit_mesh
                    // qubit outline
                    const qubit_outline_mesh = new THREE.Mesh(qubit_geometry, qubit_outline_material)
                    load_position(qubit_outline_mesh.position, display_position,)
                    update_mesh_outline(qubit_outline_mesh)
                    scene.add(qubit_outline_mesh)
                    qubit_outline_meshes[i][j] = qubit_outline_mesh
                }
            }
        }
        update_visible_qubit()
        // draw idle gates
        dispose_mesh_3d_array(idle_gate_meshes)
        idle_gate_meshes = build_3d_array(height, vertical, horizontal)
        for (let t = 0; t < height; ++t) {
            for (let i = 0; i < vertical; ++i) {
                for (let j = 0; j < horizontal; ++j) {
                    const node = nodes[t][i][j]
                    const next_node = nodes[t + 1]?.[i]?.[j]
                    if ((t == height - 1 && node != null && !node.v) || (
                        next_node != null && !next_node.v && !(next_node.gt == "InitializeX" || next_node.gt == "InitializeZ")
                    )) {
                        const position = qecp_data.simulator.positions[i][j]
                        const display_position = {
                            t: t + t_bias,  // idle gate is before every real gate
                            x: position.x,
                            y: position.y,
                        }
                        const idle_gate_mesh = new THREE.Mesh(idle_gate_geometry, idle_gate_material)
                        idle_gate_mesh.userData = {
                            type: "idle_gate",
                            t: t,
                            i: i,
                            j: j,
                            gate_peer: node.gp,
                        }
                        load_position(idle_gate_mesh.position, display_position)
                        idle_gate_mesh.scale.set(1, t_scale, 1)
                        scene.add(idle_gate_mesh)
                        idle_gate_meshes[t][i][j] = idle_gate_mesh
                    }
                }
            }
        }
        update_visible_idle_sticks()
        // draw gates
        dispose_mesh_3d_array(gate_vec_meshes)
        gate_vec_meshes = build_3d_array(height, vertical, horizontal)
        for (let t = 0; t < height; ++t) {
            for (let i = 0; i < vertical; ++i) {
                for (let j = 0; j < horizontal; ++j) {
                    const node = nodes[t][i][j]
                    if (node != null && !node.v && !node.pv && node.gt != "None") {
                        const gate_material = get_gate_material(node.gt)
                        const position = qecp_data.simulator.positions[i][j]
                        const display_position = { t: t + t_bias, x: position.x, y: position.y }
                        const gate_vec_mesh = []
                        gate_vec_meshes[t][i][j] = gate_vec_mesh
                        if (node.gt == "InitializeX" || node.gt == "InitializeZ") {
                            const gate_mesh = new THREE.Mesh(initialization_geometry, gate_material)
                            load_position(gate_mesh.position, display_position)
                            scene.add(gate_mesh)
                            gate_vec_mesh.push(gate_mesh)
                        } else if (node.gt == "MeasureX" || node.gt == "MeasureZ") {
                            if (t != 0) {  // the first measurement is always noiseless, thus no need to draw it
                                const gate_mesh = new THREE.Mesh(measurement_geometry, gate_material)
                                load_position(gate_mesh.position, display_position)
                                scene.add(gate_mesh)
                                gate_vec_mesh.push(gate_mesh)
                            }
                        } else if (node.gt == "CXGateControl" || node.gt == "CYGateControl" || node.gt == "CZGate") {
                            // dot
                            const dot_mesh = new THREE.Mesh(control_qubit_geometry, gate_material)
                            load_position(dot_mesh.position, display_position)
                            scene.add(dot_mesh)
                            gate_vec_mesh.push(dot_mesh)
                            // line
                            const peer = get_position(node.gp)
                            const peer_position = qecp_data.simulator.positions[peer.i][peer.j]
                            const peer_display_position = { t: t + t_bias, x: peer_position.x, y: peer_position.y }
                            const line_mesh = new THREE.Mesh(control_line_geometry, gate_material)
                            const relative = compute_vector3(peer_display_position).add(compute_vector3(display_position).multiplyScalar(-1))
                            const direction = relative.clone().normalize()
                            const quaternion = new THREE.Quaternion()
                            quaternion.setFromUnitVectors(unit_up_vector, direction)
                            load_position(line_mesh.position, display_position)
                            line_mesh.scale.set(1, relative.length() / 2, 1)
                            line_mesh.setRotationFromQuaternion(quaternion)
                            scene.add(line_mesh)
                            gate_vec_mesh.push(line_mesh)
                        } else if (node.gt == "CXGateTarget") {
                            // X gate
                            for (let k = 0; k < CX_target_geometries.length; ++k) {
                                const geometry = CX_target_geometries[k]
                                let mesh = new THREE.Mesh(geometry, gate_material)
                                load_position(mesh.position, display_position)
                                scene.add(mesh)
                                gate_vec_mesh.push(mesh)
                            }
                            // line
                            const peer = get_position(node.gp)
                            const peer_position = qecp_data.simulator.positions[peer.i][peer.j]
                            const peer_display_position = { t: t + t_bias, x: peer_position.x, y: peer_position.y }
                            const line_mesh = new THREE.Mesh(control_line_geometry, gate_material)
                            const relative = compute_vector3(peer_display_position).add(compute_vector3(display_position).multiplyScalar(-1))
                            const direction = relative.clone().normalize()
                            const quaternion = new THREE.Quaternion()
                            quaternion.setFromUnitVectors(unit_up_vector, direction)
                            load_position(line_mesh.position, display_position)
                            line_mesh.scale.set(1, relative.length() / 2, 1)
                            line_mesh.setRotationFromQuaternion(quaternion)
                            scene.add(line_mesh)
                            gate_vec_mesh.push(line_mesh)
                        } else if (node.gt == "CYGateTarget") {
                            // Y gate
                            for (let k = 0; k < CY_target_geometries.length; ++k) {
                                const geometry = CY_target_geometries[k]
                                let mesh = new THREE.Mesh(geometry, gate_material)
                                load_position(mesh.position, display_position)
                                scene.add(mesh)
                                gate_vec_mesh.push(mesh)
                            }
                            // line
                            const peer = get_position(node.gp)
                            const peer_position = qecp_data.simulator.positions[peer.i][peer.j]
                            const peer_display_position = { t: t + t_bias, x: peer_position.x, y: peer_position.y }
                            const relative = compute_vector3(peer_display_position).add(compute_vector3(display_position).multiplyScalar(-1))
                            const direction = relative.clone().normalize()
                            const quaternion = new THREE.Quaternion()
                            quaternion.setFromUnitVectors(unit_up_vector, direction)
                            let edge_length = relative.length() / 2 - CY_target_radius / Math.sqrt(2)
                            if (edge_length > 0) {
                                const biased_position = compute_vector3(display_position)
                                    .add(relative.clone().multiplyScalar((relative.length() / 2 - edge_length) / relative.length()))
                                // console.log(compute_vector3(display_position), biased_display_position)
                                const line_mesh = new THREE.Mesh(control_line_geometry, gate_material)
                                line_mesh.position.copy(biased_position)
                                line_mesh.scale.set(1, edge_length, 1)
                                line_mesh.setRotationFromQuaternion(quaternion)
                                scene.add(line_mesh)
                                gate_vec_mesh.push(line_mesh)
                            }
                        } else {
                            console.error(`unknown gate_type: ${node.gt}`)
                        }
                    }
                }
            }
        }
        update_visible_gates()
        // draw model graph
        dispose_mesh_3d_array(model_graph_edge_vec_meshes)
        model_graph_edge_vec_meshes = build_3d_array(height, vertical, horizontal)
        dispose_mesh_3d_array(model_graph_vertex_meshes)
        model_graph_vertex_meshes = build_3d_array(height, vertical, horizontal)
        if (qecp_data.model_graph != null) {
            existed_model_graph.value = true
            // first calculate region
            let model_graph_vertex_positions = []
            let model_graph_vertex_indices = {}
            for (let t = 0; t < height; ++t) {
                for (let i = 0; i < vertical; ++i) {
                    for (let j = 0; j < horizontal; ++j) {
                        const model_graph_node = qecp_data.model_graph.nodes[t][i][j]
                        if (model_graph_node != null) {
                            model_graph_vertex_indices[model_graph_node.p] = model_graph_vertex_positions.length
                            model_graph_vertex_positions.push(model_graph_node.p)
                        }
                    }
                }
            }
            let union_find = new UnionFind(model_graph_vertex_positions.length)
            for (let vertex_index = 0; vertex_index < model_graph_vertex_positions.length; ++vertex_index) {
                let { t, i, j } = get_position(model_graph_vertex_positions[vertex_index])
                const model_graph_node = qecp_data.model_graph.nodes[t][i][j]
                for (let [peer_position_str, edge] of Object.entries(model_graph_node.edges)) {
                    const peer_vertex_index = model_graph_vertex_indices[peer_position_str]
                    console.assert(peer_vertex_index != null)
                    union_find.union(vertex_index, peer_vertex_index)
                }
            }
            let regions = []
            let regions_union_indices = {}
            for (let vertex_index = 0; vertex_index < model_graph_vertex_positions.length; ++vertex_index) {
                const union_index = union_find.find(vertex_index)
                if (!(union_index in regions_union_indices)) {
                    regions_union_indices[union_index] = regions.length
                    regions.push(union_index)
                }
            }
            model_graph_regions.value = regions.length
            // add geometries
            for (let t = 0; t < height; ++t) {
                for (let i = 0; i < vertical; ++i) {
                    for (let j = 0; j < horizontal; ++j) {
                        const model_graph_node = qecp_data.model_graph.nodes[t][i][j]
                        if (model_graph_node != null) {
                            const position = qecp_data.simulator.positions[i][j]
                            const display_position = { t: t + t_bias, x: position.x, y: position.y }
                            const node = qecp_data.simulator.nodes[t][i][j]
                            const vertex_index = model_graph_vertex_indices[node.p]
                            const union_index = union_find.find(vertex_index)
                            const region_idx = regions_union_indices[union_index]
                            model_graph_node.region_idx = region_idx
                            // vertices
                            const vertex_mesh = new THREE.Mesh(model_graph_vertex_geometry, model_graph_vertex_material_vec[model_graph_node.region_idx])
                            load_position(vertex_mesh.position, display_position)
                            vertex_mesh.userData = {
                                type: "model_graph_vertex",
                                t: t,
                                i: i,
                                j: j,
                                region_idx: model_graph_node.region_idx,
                            }
                            scene.add(vertex_mesh)
                            model_graph_vertex_meshes[t][i][j] = vertex_mesh
                            // edges
                            const model_graph_edge_vec_mesh = []
                            model_graph_edge_vec_meshes[t][i][j] = model_graph_edge_vec_mesh
                            let vec_mesh_idx = 0
                            for (let [peer_position_str, edge] of Object.entries(model_graph_node.edges)) {
                                const { i: pi, j: pj, t: pt } = get_position(peer_position_str)
                                const peer_position = qecp_data.simulator.positions[pi][pj]
                                const peer_display_position = { t: pt + t_bias, x: peer_position.x, y: peer_position.y }
                                const line_mesh = new THREE.Mesh(model_graph_edge_geometry, model_graph_edge_material)
                                line_mesh.userData = {
                                    type: "model_graph_edge",
                                    t: t,
                                    i: i,
                                    j: j,
                                    peer: peer_position_str,
                                    edge: edge,
                                    region_idx: model_graph_node.region_idx,
                                    vec_mesh_idx: vec_mesh_idx,
                                }
                                const relative = compute_vector3(peer_display_position).add(compute_vector3(display_position).multiplyScalar(-1))
                                const direction = relative.clone().normalize()
                                const quaternion = new THREE.Quaternion()
                                quaternion.setFromUnitVectors(unit_up_vector, direction)
                                load_position(line_mesh.position, display_position)
                                line_mesh.scale.set(1, relative.length() / 2, 1)
                                line_mesh.setRotationFromQuaternion(quaternion)
                                scene.add(line_mesh)
                                model_graph_edge_vec_mesh.push(line_mesh)
                                vec_mesh_idx += 1
                            }
                            // there might be some duplicate boundary edges, deduplicate them
                            let boundary_peers = {}
                            for (let [boundary_idx, boundary] of model_graph_node.all_boundaries.entries()) {
                                let [vpi, vpj, vpt] = [i, j, t - qecp_data.simulator.measurement_cycles]
                                if (boundary.v != null) {
                                    const vp = get_position(boundary.v)
                                    vpi = vp.i; vpj = vp.j; vpt = vp.t
                                }
                                const id = `[${vpt}][${vpi}][${vpj}]`
                                if (boundary_peers[id] != null) {
                                    continue
                                }
                                boundary_peers[id] = true  // mark exist
                                const peer_position = qecp_data.simulator.positions[vpi][vpj]
                                const peer_display_position = { t: vpt + t_bias, x: peer_position.x, y: peer_position.y }
                                const line_mesh = new THREE.Mesh(model_graph_edge_geometry, model_graph_edge_material)
                                line_mesh.userData = {
                                    type: "model_graph_boundary",
                                    t: t,
                                    i: i,
                                    j: j,
                                    boundary_idx: boundary_idx,
                                    boundary: boundary,
                                    region_idx: model_graph_node.region_idx,
                                    vec_mesh_idx: vec_mesh_idx,
                                }
                                const relative = compute_vector3(peer_display_position).add(compute_vector3(display_position).multiplyScalar(-1))
                                const direction = relative.clone().normalize()
                                const quaternion = new THREE.Quaternion()
                                quaternion.setFromUnitVectors(unit_up_vector, direction)
                                load_position(line_mesh.position, display_position)
                                line_mesh.scale.set(1, relative.length(), 1)
                                line_mesh.setRotationFromQuaternion(quaternion)
                                scene.add(line_mesh)
                                model_graph_edge_vec_mesh.push(line_mesh)
                                vec_mesh_idx += 1
                            }
                        }
                    }
                }
            }
            let default_region_display = []
            for (let i = 0; i < model_graph_regions.value; ++i) default_region_display.push(i)
            model_graph_region_display.value = default_region_display
        } else {
            display_model_graph.value = null  // show intermediate state
        }
        update_visible_model_graph()
        // draw model hypergraph
        if (qecp_data.model_hypergraph != null) {
            existed_model_hypergraph.value = true
            // add geometry for each vertex in the model hypergraph
            dispose_mesh_1d_array(model_hypergraph_vertex_meshes)
            model_hypergraph_vertex_meshes = []
            qecp_data.model_hypergraph.incident_edges = []  // compute incident edges for each vertex for visualization
            for (let vertex_index = 0; vertex_index < qecp_data.model_hypergraph.vertex_positions.length; ++vertex_index) {
                qecp_data.model_hypergraph.incident_edges.push([])
                let position_str = qecp_data.model_hypergraph.vertex_positions[vertex_index]
                const tij = get_position(position_str)
                const position = qecp_data.simulator.positions[tij.i][tij.j]
                const display_position = { t: tij.t + t_bias, x: position.x, y: position.y }
                const vertex_mesh = new THREE.Mesh(model_graph_vertex_geometry, model_graph_vertex_material_vec[1])
                load_position(vertex_mesh.position, display_position)
                vertex_mesh.userData = {
                    type: "model_hypergraph_vertex",
                    vertex_index: vertex_index,
                }
                scene.add(vertex_mesh)
                model_hypergraph_vertex_meshes.push(vertex_mesh)
            }
            // add geometries for each hyperedge
            dispose_mesh_1d_array(model_hypergraph_edge_vec_meshes)
            model_hypergraph_edge_vec_meshes = []
            qecp_data.model_hypergraph.edge_min_max_t = []
            for (let edge_index = 0; edge_index < qecp_data.model_hypergraph.weighted_edges.length; ++edge_index) {
                let [defect_vertices_str, hyperedge_group] = qecp_data.model_hypergraph.weighted_edges[edge_index]
                const defect_vertices = get_defect_vertices(defect_vertices_str)
                let edge_vec_mesh = []
                // calculate hyperedge center
                let sum_position = new THREE.Vector3(0, 0, 0)
                let visual_positions = []
                let max_t = null
                let min_t = null
                for (let j = 0; j < defect_vertices.length; ++j) {
                    const tij = defect_vertices[j]
                    if (j == 0) max_t = tij.t
                    if (j == 0) min_t = tij.t
                    if (tij.t > max_t) max_t = tij.t
                    if (tij.t < min_t) min_t = tij.t
                    const position = qecp_data.simulator.positions[tij.i][tij.j]
                    const display_position = { t: tij.t + t_bias, x: position.x, y: position.y }
                    let visual_position = compute_vector3(display_position)
                    visual_positions.push(visual_position)
                    // console.log(visual_position)
                    sum_position = sum_position.add(visual_position)
                    const vertex_index = qecp_data.model_hypergraph.vertex_indices[to_position_str(tij)]
                    qecp_data.model_hypergraph.incident_edges[vertex_index].push(edge_index)
                }
                qecp_data.model_hypergraph.edge_min_max_t.push([min_t, max_t])
                const center_position = sum_position.multiplyScalar(1 / defect_vertices.length)
                for (let j = 0; j < defect_vertices.length; ++j) {
                    const edge_mesh = new THREE.Mesh(get_hyperedge_geometry(defect_vertices.length), model_hypergraph_edge_material)
                    edge_mesh.userData = {
                        type: "model_hypergraph_edge",
                        edge_index: edge_index,
                    }
                    let visual_position = visual_positions[j]
                    const relative = center_position.clone().add(visual_position.clone().multiplyScalar(-1))
                    const direction = relative.clone().normalize()
                    // console.log(direction)
                    const quaternion = new THREE.Quaternion()
                    quaternion.setFromUnitVectors(unit_up_vector, direction)
                    const distance = relative.length()
                    edge_mesh.position.copy(visual_position)
                    if (defect_vertices.length != 1) {
                        edge_mesh.scale.set(1, distance, 1)
                        edge_mesh.setRotationFromQuaternion(quaternion)
                    }
                    edge_mesh.visible = true
                    if (defect_vertices.length != 1 && distance == 0) {
                        edge_mesh.visible = false
                    }
                    scene.add(edge_mesh)
                    edge_vec_mesh.push(edge_mesh)
                }
                model_hypergraph_edge_vec_meshes.push(edge_vec_mesh)
            }
        }
        update_visible_model_hypergraph()
        // draw tailored model graph
        dispose_mesh_3d_array(tailored_model_graph_edge_vec_meshes)
        tailored_model_graph_edge_vec_meshes = build_3d_array(height, vertical, horizontal)
        dispose_mesh_3d_array(tailored_model_graph_vertex_meshes)
        tailored_model_graph_vertex_meshes = build_3d_array(height, vertical, horizontal)
        if (qecp_data.tailored_model_graph != null) {
            existed_tailored_model_graph.value = true
            // add geometries
            for (let t = 0; t < height; ++t) {
                for (let i = 0; i < vertical; ++i) {
                    for (let j = 0; j < horizontal; ++j) {
                        const triple_tailored_node = qecp_data.tailored_model_graph.nodes[t][i][j]
                        if (triple_tailored_node != null) {
                            const position = qecp_data.simulator.positions[i][j]
                            const display_position = { t: t + t_bias, x: position.x, y: position.y }
                            // vertices
                            const vertex_mesh = new THREE.Mesh(
                                model_graph_vertex_geometry,
                                tailored_model_graph_vertex_material
                            )
                            load_position(vertex_mesh.position, display_position)
                            vertex_mesh.userData = {
                                type: "tailored_vertex",
                                t: t,
                                i: i,
                                j: j,
                                color: "black",
                            }
                            scene.add(vertex_mesh)
                            tailored_model_graph_vertex_meshes[t][i][j] = vertex_mesh
                            // edges
                            const tailored_model_graph_edge_vec_mesh = []
                            tailored_model_graph_edge_vec_meshes[t][i][j] = tailored_model_graph_edge_vec_mesh
                            let vec_mesh_idx = 0
                            for (let tsg = 0; tsg < 3; ++tsg) {  // tailored subgraph
                                const tailored_node = triple_tailored_node[tsg]
                                for (let [peer_position_str, edge] of Object.entries(tailored_node.edges)) {
                                    const { i: pi, j: pj, t: pt } = get_position(peer_position_str)
                                    const peer_position = qecp_data.simulator.positions[pi][pj]
                                    const peer_display_position = { t: pt + t_bias, x: peer_position.x, y: peer_position.y }
                                    const line_mesh = new THREE.Mesh(
                                        model_graph_edge_geometry,
                                        tailored_model_graph_edge_material_vec[tsg]
                                    )
                                    line_mesh.userData = {
                                        type: "tailored_edge",
                                        t: t,
                                        i: i,
                                        j: j,
                                        peer: peer_position_str,
                                        edge: edge,
                                        tsg: tsg,
                                        vec_mesh_idx: vec_mesh_idx,
                                    }
                                    const relative = compute_vector3(peer_display_position).add(compute_vector3(display_position).multiplyScalar(-1))
                                    const direction = relative.clone().normalize()
                                    const quaternion = new THREE.Quaternion()
                                    quaternion.setFromUnitVectors(unit_up_vector, direction)
                                    load_position(line_mesh.position, display_position)
                                    line_mesh.scale.set(1, relative.length() / 2, 1)
                                    line_mesh.setRotationFromQuaternion(quaternion)
                                    scene.add(line_mesh)
                                    tailored_model_graph_edge_vec_mesh.push(line_mesh)
                                    vec_mesh_idx += 1
                                }
                            }
                        }
                    }
                }
            }
            // modify materials
            for (const position_str of qecp_data.tailored_model_graph.virtual_nodes) {
                const { t, i, j } = get_position(position_str)
                const vertex_mesh = tailored_model_graph_vertex_meshes[t][i][j]
                vertex_mesh.material = tailored_model_graph_virtual_vertex_material
                vertex_mesh.userData.is_virtual = true
                vertex_mesh.userData.color = "orange"
            }
            for (const position_str_pair of qecp_data.tailored_model_graph.corner_virtual_nodes) {
                for (let k = 0; k < 2; ++k) {
                    const { t, i, j } = get_position(position_str_pair[k])
                    const vertex_mesh = tailored_model_graph_vertex_meshes[t][i][j]
                    vertex_mesh.material = tailored_model_graph_corner_vertex_material
                    vertex_mesh.userData.is_corner = true
                    vertex_mesh.userData.corner_pair = get_position(position_str_pair[(k + 1) % 2])
                    vertex_mesh.userData.color = "red"
                }
            }
        }
        // draw noise model: Pauli and erasure errors probabilities
        dispose_mesh_3d_array(noise_model_pauli_meshes)
        noise_model_pauli_meshes = build_3d_array(height, vertical, horizontal)
        dispose_mesh_3d_array(noise_model_erasure_meshes)
        noise_model_erasure_meshes = build_3d_array(height, vertical, horizontal)
        contributed_noise_sources = build_3d_array(height, vertical, horizontal, () => [])
        if (qecp_data.noise_model != null) {
            existed_noise_model.value = true
            // first calculate whether to display it
            function sum_error_rate(obj) {
                if (obj == null) return 0
                let sum = 0
                if (obj != null) {
                    for (const name in obj) {
                        sum += parseFloat(obj[name])
                    }
                }
                return sum
            }
            // iterate through in-place noises
            for (let t = 0; t < height; ++t) {
                for (let i = 0; i < vertical; ++i) {
                    for (let j = 0; j < horizontal; ++j) {
                        const noise_model_node = qecp_data.noise_model.nodes[t][i][j]
                        if (noise_model_node != null) {
                            // 1e-300 is typically used for supporting decoding erasure errors or mimic infinitely small error rate
                            if (noise_model_node.pp.px > 2e-300 || noise_model_node.pp.pz > 2e-300 || noise_model_node.pp.py > 2e-300) {
                                noise_model_node.display_pauli = true
                                contributed_noise_sources[t][i][j].push({ source: "pp", t, i, j })
                            }
                            if (noise_model_node.pe > 2e-300) {
                                noise_model_node.display_erasure = true
                                contributed_noise_sources[t][i][j].push({ source: "pe", t, i, j })
                            }
                            // correlated ones
                            const node = qecp_data.simulator.nodes[t][i][j]
                            const gate_peer = node.gp
                            let sum_correlated_pauli = sum_error_rate(noise_model_node.corr_pp)
                            if (sum_correlated_pauli > 0) {
                                noise_model_node.display_pauli = true
                                contributed_noise_sources[t][i][j].push({ source: "corr_pp", t, i, j })
                                if (gate_peer != null) {
                                    const peer = get_position(gate_peer)
                                    qecp_data.noise_model.nodes[peer.t][peer.i][peer.j].display_pauli = true
                                    contributed_noise_sources[peer.t][peer.i][peer.j].push({ source: "corr_pp", others: true, t, i, j })
                                }
                            }
                            let sum_correlated_erasure = sum_error_rate(noise_model_node.corr_pe)
                            if (sum_correlated_erasure > 0) {
                                noise_model_node.display_erasure = true
                                contributed_noise_sources[t][i][j].push({ source: "corr_pe", t, i, j })
                                if (gate_peer != null) {
                                    const peer = get_position(gate_peer)
                                    qecp_data.noise_model.nodes[peer.t][peer.i][peer.j].display_erasure = true
                                    contributed_noise_sources[peer.t][peer.i][peer.j].push({ source: "corr_pe", others: true, t, i, j })
                                }
                            }
                        }
                    }
                }
            }
            // iterate through additional noises
            for (let ai = 0; ai < qecp_data.noise_model.additional_noise.length; ++ai) {
                const noise = qecp_data.noise_model.additional_noise[ai]
                if (noise.p > 2e-300) {
                    for (const position_str in noise.pe) {  // pe: { pos1: err1, pos2: err2 }
                        const { t, i, j } = get_position(position_str)
                        const noise_model_node = qecp_data.noise_model.nodes[t][i][j]
                        noise_model_node.display_pauli = true
                        contributed_noise_sources[t][i][j].push({ source: "add_p", others: true, add_idx: ai, t, i, j })
                    }
                    for (const position_str of noise.ee) {  // ee: [pos1, pos2]
                        const { t, i, j } = get_position(position_str)
                        const noise_model_node = qecp_data.noise_model.nodes[t][i][j]
                        noise_model_node.display_erasure = true
                        contributed_noise_sources[t][i][j].push({ source: "add_e", others: true, add_idx: ai, t, i, j })
                    }
                }
            }
            // add geometries
            for (let t = 0; t < height; ++t) {
                for (let i = 0; i < vertical; ++i) {
                    for (let j = 0; j < horizontal; ++j) {
                        const node = qecp_data.simulator.nodes[t][i][j]
                        const noise_model_node = qecp_data.noise_model.nodes[t][i][j]
                        if (noise_model_node != null) {
                            const position = qecp_data.simulator.positions[i][j]
                            const display_position = { t: t + t_bias, x: position.x, y: position.y }
                            // non-zero Pauli errors
                            if (noise_model_node.display_pauli) {
                                const pauli_mesh = new THREE.Mesh(noise_model_pauli_geometry, noise_model_pauli_material)
                                load_position(pauli_mesh.position, display_position)
                                pauli_mesh.userData = {
                                    type: "noise_model_node_pauli",
                                    t: t,
                                    i: i,
                                    j: j,
                                    gate_peer: node.gp,
                                }
                                scene.add(pauli_mesh)
                                noise_model_pauli_meshes[t][i][j] = pauli_mesh
                            }
                            // non-zero erasure errors
                            if (noise_model_node.display_erasure) {
                                const erasure_mesh = new THREE.Mesh(noise_model_erasure_geometry, noise_model_erasure_material)
                                load_position(erasure_mesh.position, display_position)
                                erasure_mesh.userData = {
                                    type: "noise_model_node_erasure",
                                    t: t,
                                    i: i,
                                    j: j,
                                    gate_peer: node.gp,
                                }
                                scene.add(erasure_mesh)
                                noise_model_erasure_meshes[t][i][j] = erasure_mesh
                            }
                        }
                    }
                }
            }
        }
        update_visible_noise_model()
        // refresh active case as well
        await refresh_case()
        // if provided in url, then apply the selection
        for (let i = 0; i < 3; ++i) await Vue.nextTick()
        current_selected.value = JSON.parse(urlParams.get('current_selected'))
    }
}

export const log_matchings_name_vec = ref([])
export const log_matchings_display = ref([])
export const display_matchings = ref(true)
export function update_visible_matchings() {
    const active_matchings = {}
    for (const matching_index of Object.values(log_matchings_display.value)) active_matchings[matching_index] = true
    for (let i = 0; i < matching_meshes.length; i++) {
        const mesh_vec = matching_meshes[i]
        for (let j = 0; j < mesh_vec.length; j++) {
            const edge_mesh = mesh_vec[j]
            const is_active = display_matchings.value && active_matchings[i] == true
            const in_range = in_t_range_any(edge_mesh.userData.t1, edge_mesh.userData.t2)
            edge_mesh.visible = is_active && in_range
        }
    }
}
watch([log_matchings_display, display_matchings, t_range], update_visible_matchings, { deep: true })

export const active_qecp_data = shallowRef(null)
export const active_case_idx = ref(0)
export async function refresh_case() {
    // console.log("refresh_case")
    if (active_qecp_data.value != null) {  // no qecp data provided
        const qecp_data = active_qecp_data.value
        const case_idx = active_case_idx.value
        const active_case = qecp_data.cases[case_idx]
        // clear hover and select
        current_hover.value = null
        let current_selected_value = JSON.parse(JSON.stringify(current_selected.value))
        current_selected.value = null
        await Vue.nextTick()
        await Vue.nextTick()
        // constants
        const height = qecp_data.simulator.height
        const t_bias = -height / 2
        // draw measurements
        dispose_mesh_1d_array(defect_measurement_meshes)
        dispose_mesh_1d_array(defect_measurement_outline_meshes)
        defect_measurement_meshes = []
        defect_measurement_outline_meshes = []
        for (let defect_idx = 0; defect_idx < active_case.measurement.length; ++defect_idx) {
            const defect_position = active_case.measurement[defect_idx]
            const { t, i, j } = get_position(defect_position)
            const position = qecp_data.simulator.positions[i][j]
            const display_position = {
                t: t + t_bias,
                x: position.x,
                y: position.y,
            }
            // defect measurement
            const defect_measurement_mesh = new THREE.Mesh(defect_measurement_geometry, defect_measurement_material)
            defect_measurement_mesh.userData = {
                type: "defect",
                defect_idx: defect_idx,
                t: t,
                i: i,
                j: j,
            }
            scene.add(defect_measurement_mesh)
            load_position(defect_measurement_mesh.position, display_position)
            defect_measurement_meshes.push(defect_measurement_mesh)
            // defect measurement outline
            const defect_measurement_outline_mesh = new THREE.Mesh(defect_measurement_geometry, defect_measurement_outline_material)
            load_position(defect_measurement_outline_mesh.position, display_position,)
            update_mesh_outline(defect_measurement_outline_mesh)
            scene.add(defect_measurement_outline_mesh)
            defect_measurement_outline_meshes.push(defect_measurement_outline_mesh)
        }
        update_visible_defect_measurement()
        // draw error pattern and detected erasures
        dispose_mesh_1d_array(error_pattern_vec_meshes)
        error_pattern_vec_meshes = []
        dispose_mesh_1d_array(detected_erasure_meshes)
        detected_erasure_meshes = []
        for (let [idx, [position_str, error]] of Object.entries(active_case.error_pattern).entries()) {
            const { t, i, j } = get_position(position_str)
            const position = qecp_data.simulator.positions[i][j]
            const display_position = {
                t: t + t_bias + 0.5,
                x: position.x,
                y: position.y,
            }
            const error_pattern_vec_mesh = []
            error_pattern_vec_meshes.push(error_pattern_vec_mesh)
            let error_geometries = []
            if (error == "X") {
                error_geometries = error_X_geometries
            } else if (error == "Y") {
                error_geometries = error_Y_geometries
            } else if (error == "Z") {
                error_geometries = error_Z_geometries
            } else if (error == "I") { } else {
                console.error(`unknown error type: ${error}`)
            }
            for (let k = 0; k < error_geometries.length; ++k) {
                const geometry = error_geometries[k]
                let mesh = new THREE.Mesh(geometry, error_materials[error])
                load_position(mesh.position, display_position)
                mesh.userData = {
                    type: "error",
                    idx: idx,
                    t: t,
                    i: i,
                    j: j,
                }
                scene.add(mesh)
                error_pattern_vec_mesh.push(mesh)
            }
        }
        if (active_case.detected_erasures) {
            for (let [idx, position_str] of Object.entries(active_case.detected_erasures)) {
                const { t, i, j } = get_position(position_str)
                const position = qecp_data.simulator.positions[i][j]
                const display_position = {
                    t: t + t_bias + 0.5,
                    x: position.x,
                    y: position.y,
                }
                let mesh = new THREE.Mesh(detected_erasure_geometry, detected_erasure_material)
                load_position(mesh.position, display_position)
                mesh.userData = {
                    type: "erasure",
                    idx: idx,
                    t: t,
                    i: i,
                    j: j,
                }
                scene.add(mesh)
                detected_erasure_meshes.push(mesh)
            }
        }
        update_visible_error_pattern()
        // draw correction
        dispose_mesh_1d_array(correction_vec_meshes)
        correction_vec_meshes = []
        if (active_case.correction) {
            for (let [idx, [position_str, error]] of Object.entries(active_case.correction).entries()) {
                const { t, i, j } = get_position(position_str)
                const position = qecp_data.simulator.positions[i][j]
                const display_position = {
                    t: t + t_bias + 1,
                    x: position.x,
                    y: position.y,
                }
                const correction_vec_mesh = []
                correction_vec_meshes.push(correction_vec_mesh)
                let error_geometries = []
                if (error == "X") {
                    error_geometries = error_X_geometries
                } else if (error == "Y") {
                    error_geometries = error_Y_geometries
                } else if (error == "Z") {
                    error_geometries = error_Z_geometries
                } else if (error == "I") { } else {
                    console.error(`unknown error type: ${error}`)
                }
                for (let k = 0; k < error_geometries.length; ++k) {
                    const geometry = error_geometries[k]
                    let mesh = new THREE.Mesh(geometry, error_materials[error])
                    load_position(mesh.position, display_position)
                    mesh.userData = {
                        type: "correction",
                        idx: idx,
                        t: t,
                        i: i,
                        j: j,
                    }
                    scene.add(mesh)
                    correction_vec_mesh.push(mesh)
                }
            }
        }
        update_visible_correction()
        // draw matchings
        dispose_mesh_2d_array(matching_meshes)
        matching_meshes = []
        log_matchings_name_vec.value = []
        if (active_case.runtime_statistics?.log_matchings) {
            const name_vec = []
            const log_matchings = active_case.runtime_statistics.log_matchings
            for (let i = 0; i < log_matchings.length; i++) {
                const log_matching = log_matchings[i]
                name_vec.push({
                    name: log_matching.name,
                    desc: log_matching.description,
                })
                const mesh_vec = []
                for (let j = 0; j < log_matching.edges.length; ++j) {
                    const [position_str_1, position_str_2] = log_matching.edges[j]
                    const { t: t1, i: i1, j: j1 } = get_position(position_str_1)
                    const { t: t2, i: i2, j: j2 } = get_position(position_str_2)
                    const position_1 = qecp_data.simulator.positions[i1][j1]
                    const position_2 = qecp_data.simulator.positions[i2][j2]
                    const display_position_1 = { t: t1 + t_bias, x: position_1.x, y: position_1.y }
                    const display_position_2 = { t: t2 + t_bias, x: position_2.x, y: position_2.y }
                    const line_mesh = new THREE.Mesh(
                        matching_edge_geometry,
                        model_graph_vertex_material_vec[i]
                    )
                    line_mesh.userData = {
                        type: "matching_edge",
                        t1, t2,
                        matching_idx: i,
                        edge_idx: j,
                    }
                    const relative = compute_vector3(display_position_2).add(compute_vector3(display_position_1).multiplyScalar(-1))
                    const direction = relative.clone().normalize()
                    const quaternion = new THREE.Quaternion()
                    quaternion.setFromUnitVectors(unit_up_vector, direction)
                    load_position(line_mesh.position, display_position_1)
                    line_mesh.scale.set(1, relative.length(), 1)
                    line_mesh.setRotationFromQuaternion(quaternion)
                    scene.add(line_mesh)
                    mesh_vec.push(line_mesh)
                }
                matching_meshes.push(mesh_vec)
            }
            log_matchings_name_vec.value = name_vec
        }
        update_visible_matchings()
        // reset select
        await Vue.nextTick()
        if (is_user_data_valid(current_selected_value)) {
            current_selected.value = current_selected_value
        }
    }
}
watch([active_qecp_data], refresh_qecp_data)  // call refresh_case
watch([active_case_idx], refresh_case)
export function show_case(case_idx, qecp_data) {
    active_case_idx.value = case_idx
    active_qecp_data.value = qecp_data
}

// configurations
const gui = new GUI({ width: 400, title: "render configurations" })
export const show_config = ref(false)
watch(show_config, () => {
    if (show_config.value) {
        gui.domElement.style.display = "block"
    } else {
        gui.domElement.style.display = "none"
    }
}, { immediate: true })
watch(sizes, () => {  // move render configuration GUI to 3D canvas
    // gui.domElement.style.right = sizes.control_bar_width + "px"
    gui.domElement.style.right = 0
}, { immediate: true })
const conf = {
    scene_background: scene.background,
    outline_ratio: outline_ratio.value,
    qubit_radius_scale: qubit_radius_scale.value,
    idle_gate_radius_scale: idle_gate_radius_scale.value,
    defect_measurement_radius_scale: defect_measurement_radius_scale.value,
}
const side_options = { "FrontSide": THREE.FrontSide, "BackSide": THREE.BackSide, "DoubleSide": THREE.DoubleSide }
export const controller = {}
window.controller = controller
controller.scene_background = gui.addColor(conf, 'scene_background').onChange(
    function (value) { scene.background = value })
const size_folder = gui.addFolder('size')
controller.outline_ratio = size_folder.add(conf, 'outline_ratio', 0.99, 2).onChange(
    function (value) { outline_ratio.value = Number(value) })
controller.qubit_radius_scale = size_folder.add(conf, 'qubit_radius_scale', 0.1, 5).onChange(
    function (value) { qubit_radius_scale.value = Number(value) })
controller.idle_gate_radius_scale = size_folder.add(conf, 'idle_gate_radius_scale', 0.1, 10).onChange(
    function (value) { idle_gate_radius_scale.value = Number(value) })
controller.defect_measurement_radius_scale = size_folder.add(conf, 'defect_measurement_radius_scale', 0.1, 10).onChange(
    function (value) { defect_measurement_radius_scale.value = Number(value) })
watch(sizes, () => {
    gui.domElement.style.transform = `scale(${sizes.scale})`
    gui.domElement.style["transform-origin"] = "right top"
}, { immediate: true })

// select logic
const raycaster = new THREE.Raycaster()
const mouse = new THREE.Vector2()
var previous_hover_material = null
export const current_hover = shallowRef(null)
window.current_hover = current_hover
var previous_selected_material = null
export const current_selected = shallowRef(null)
window.current_selected = current_selected
export const show_hover_effect = ref(true)
function is_user_data_valid(user_data) {
    const qecp_data = active_qecp_data.value
    const case_idx = active_case_idx.value
    if (user_data == null || qecp_data == null || case_idx == null) return false
    const active_case = qecp_data.cases[case_idx][1]
    // constants
    const height = qecp_data.simulator.height
    const vertical = qecp_data.simulator.vertical
    const horizontal = qecp_data.simulator.horizontal
    if (user_data.type == "qubit") {
        const { i, j } = user_data
        return i < vertical && j < horizontal && qecp_data.simulator.nodes[0][i][j] != null
    }
    if (user_data.type == "idle_gate") {
        const { t, i, j } = user_data
        return t < height && i < vertical && j < horizontal && qecp_data.simulator.nodes[t][i][j] != null
    }
    if (user_data.type == "noise_model_node_pauli") {
        const { t, i, j } = user_data
        return t < height && i < vertical && j < horizontal && qecp_data.simulator.nodes[t][i][j] != null
    }
    if (user_data.type == "noise_model_node_erasure") {
        const { t, i, j } = user_data
        return t < height && i < vertical && j < horizontal && qecp_data.simulator.nodes[t][i][j] != null
    }
    // defect, correction, ... are random between cases, no need to recover
    return false
}
function set_material_with_user_data(user_data, material) {  // return the previous material
    if (user_data.type == "qubit") {
        const { i, j } = user_data
        const mesh = qubit_meshes[i][j]
        let previous_material = mesh.material
        mesh.material = material
        return previous_material
    }
    if (user_data.type == "idle_gate") {
        const { t, i, j } = user_data
        const mesh = idle_gate_meshes[t][i][j]
        let previous_material = mesh.material
        mesh.material = material
        return previous_material
    }
    if (user_data.type == "noise_model_node_pauli") {
        const { t, i, j } = user_data
        const mesh = noise_model_pauli_meshes[t][i][j]
        let previous_material = mesh.material
        mesh.material = material
        return previous_material
    }
    if (user_data.type == "noise_model_node_erasure") {
        const { t, i, j } = user_data
        const mesh = noise_model_erasure_meshes[t][i][j]
        let previous_material = mesh.material
        mesh.material = material
        return previous_material
    }
    if (user_data.type == "defect") {
        const { defect_idx, t, i, j } = user_data
        const mesh = defect_measurement_meshes[defect_idx]
        let previous_material = mesh.material
        mesh.material = material
        return previous_material
    }
    if (user_data.type == "correction") {
        const { idx, t, i, j } = user_data
        const vec_mesh = correction_vec_meshes[idx]
        let previous_material = vec_mesh.map(x => x.material)
        let expanded_material = material
        if (!Array.isArray(material)) expanded_material = Array(vec_mesh.length).fill(material)
        Object.entries(expanded_material).map(([idx, material]) => { vec_mesh[idx].material = material })
        return previous_material
    }
    if (user_data.type == "model_hypergraph_vertex") {
        const { vertex_index } = user_data
        let vertex_mesh = model_hypergraph_vertex_meshes[vertex_index]
        let previous_material = vertex_mesh.material
        vertex_mesh.material = material
        return previous_material
    }
    if (user_data.type == "model_hypergraph_edge") {
        const { edge_index } = user_data
        let edge_vec_mesh = model_hypergraph_edge_vec_meshes[edge_index]
        let previous_material = edge_vec_mesh[0].material
        for (let mesh of edge_vec_mesh) {
            mesh.material = material
        }
        return previous_material
    }
    if (user_data.type == "model_graph_edge" || user_data.type == "model_graph_boundary") {
        const { t, i, j, vec_mesh_idx } = user_data
        let mesh = model_graph_edge_vec_meshes[t][i][j][vec_mesh_idx]
        let previous_material = mesh.material
        mesh.material = material
        return previous_material
    }
    if (user_data.type == "model_graph_vertex") {
        const { t, i, j } = user_data
        let mesh = model_graph_vertex_meshes[t][i][j]
        let previous_material = mesh.material
        mesh.material = material
        return previous_material
    }
    if (user_data.type == "tailored_vertex") {
        const { t, i, j } = user_data
        let mesh = tailored_model_graph_vertex_meshes[t][i][j]
        let previous_material = mesh.material
        mesh.material = material
        return previous_material
    }
    if (user_data.type == "tailored_edge") {
        const { t, i, j, vec_mesh_idx } = user_data
        let mesh = tailored_model_graph_edge_vec_meshes[t][i][j][vec_mesh_idx]
        let previous_material = mesh.material
        mesh.material = material
        return previous_material
    }
    console.error(`unknown type ${user_data.type}`)
}
watch(current_hover, (newVal, oldVal) => {
    // console.log(`${oldVal} -> ${newVal}`)
    if (oldVal != null && previous_hover_material != null) {
        set_material_with_user_data(oldVal, previous_hover_material)
        previous_hover_material = null
    }
    if (newVal != null) {
        previous_hover_material = set_material_with_user_data(newVal, hover_material)
    }
})
watch(current_selected, (newVal, oldVal) => {
    if (newVal != null) {
        current_hover.value = null
    }
    Vue.nextTick(() => {  // wait after hover cleaned its data
        if (oldVal != null && previous_selected_material != null) {
            set_material_with_user_data(oldVal, previous_selected_material)
            previous_selected_material = null
        }
        if (newVal != null) {
            previous_selected_material = set_material_with_user_data(newVal, selected_material)
        }
    })
})
function on_mouse_change(event, is_click) {
    mouse.x = (event.clientX / sizes.canvas_width) * 2 - 1
    mouse.y = - (event.clientY / sizes.canvas_height) * 2 + 1
    raycaster.setFromCamera(mouse, camera.value)
    const intersects = raycaster.intersectObjects(scene.children, false)
    for (let intersect of intersects) {
        if (!intersect.object.visible) continue  // don't select invisible object
        let user_data = intersect.object.userData
        if (user_data.type == null) continue  // doesn't contain enough information
        // swap back to the original material
        if (is_click) {
            current_selected.value = user_data
        } else {
            if (show_hover_effect.value) {
                current_hover.value = user_data
            } else {
                current_hover.value = null
            }
        }
        return
    }
    if (is_click) {
        current_selected.value = null
    } else {
        current_hover.value = null
    }
    return
}
var mousedown_position = null
var is_mouse_currently_down = false
window.addEventListener('mousedown', (event) => {
    if (event.clientX > sizes.canvas_width) return  // don't care events on control panel
    mousedown_position = {
        clientX: event.clientX,
        clientY: event.clientY,
    }
    is_mouse_currently_down = true
})
window.addEventListener('mouseup', (event) => {
    if (event.clientX > sizes.canvas_width) return  // don't care events on control panel
    // to prevent triggering select while moving camera
    if (mousedown_position != null && mousedown_position.clientX == event.clientX && mousedown_position.clientY == event.clientY) {
        on_mouse_change(event, true)
    }
    is_mouse_currently_down = false
})
window.addEventListener('mousemove', (event) => {
    if (event.clientX > sizes.canvas_width) return  // don't care events on control panel
    // to prevent triggering hover while moving camera
    if (!is_mouse_currently_down) {
        on_mouse_change(event, false)
    }
})

// export current scene to high-resolution png, useful when generating figures for publication
// (I tried svg renderer but it doesn't work very well... shaders are poorly supported)
export function render_png(scale = 1) {
    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, preserveDrawingBuffer: true, context: webgl_renderer_context() })
    renderer.setSize(sizes.canvas_width * scale, sizes.canvas_height * scale, false)
    renderer.setPixelRatio(window.devicePixelRatio * scale)
    renderer.render(scene, camera.value)
    return renderer.domElement.toDataURL()
}
window.render_png = render_png
export function open_png(data_url) {
    const w = window.open('', '')
    w.document.title = "rendered image"
    w.document.body.style.backgroundColor = "white"
    w.document.body.style.margin = "0"
    const img = new Image()
    img.src = data_url
    img.style = "width: 100%; height: 100%; object-fit: contain;"
    w.document.body.appendChild(img)
}
window.open_png = open_png
export function download_png(data_url) {
    const a = document.createElement('a')
    a.href = data_url.replace("image/png", "image/octet-stream")
    a.download = 'rendered.png'
    a.click()
}
window.download_png = download_png

export async function nodejs_render_png() {  // works only in nodejs
    let context = webgl_renderer_context()
    var pixels = new Uint8Array(context.drawingBufferWidth * context.drawingBufferHeight * 4)
    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: false, preserveDrawingBuffer: true, context })
    renderer.setSize(sizes.canvas_width, sizes.canvas_height, false)
    renderer.setPixelRatio(window.devicePixelRatio)
    renderer.render(scene, camera.value)
    context.readPixels(0, 0, context.drawingBufferWidth, context.drawingBufferHeight, context.RGBA, context.UNSIGNED_BYTE, pixels)
    return pixels
}

// wait several Vue ticks to make sure all changes have been applied
export async function wait_changes() {
    for (let i = 0; i < 5; ++i) await Vue.nextTick()
}

// https://www.npmjs.com/package/base64-arraybuffer
var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'
function base64_encode(arraybuffer) {
    var bytes = new Uint8Array(arraybuffer), i, len = bytes.length, base64 = ''
    for (i = 0; i < len; i += 3) {
        base64 += chars[bytes[i] >> 2]
        base64 += chars[((bytes[i] & 3) << 4) | (bytes[i + 1] >> 4)]
        base64 += chars[((bytes[i + 1] & 15) << 2) | (bytes[i + 2] >> 6)]
        base64 += chars[bytes[i + 2] & 63]
    }
    if (len % 3 === 2) {
        base64 = base64.substring(0, base64.length - 1) + '='
    }
    else if (len % 3 === 1) {
        base64 = base64.substring(0, base64.length - 2) + '=='
    }
    return base64;
}

// https://javascript.plainenglish.io/union-find-97f0036dff93
class UnionFind {
    constructor(N) {
        this.parent = Array.from({ length: N }, (_, i) => i)
        this.count = new Array(N).fill(1)
    }
    find(x) {
        if (this.parent[x] != x) this.parent[x] = this.find(this.parent[x])
        return this.parent[x]
    }
    union(x, y) {
        const xp = this.find(x), yp = this.find(y)
        if (xp == yp) return
        if (this.count[xp] < this.count[yp]) {
            this.parent[xp] = yp
            this.count[yp] += this.count[xp]
        } else {
            this.parent[yp] = xp
            this.count[xp] += this.count[yp]
        }
    }
}
