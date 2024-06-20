import * as gui3d from './gui3d.js'
import * as THREE from 'three'
const { ref, reactive, watch, computed } = Vue

export async function micro_blossom_paper_stabilizer_circuit() {
    this.lock_view = true
    await gui3d.wait_changes() // make sure all changes have been applied
    gui3d.camera.value.position.set(
        728.7012182647289,
        525.06404156976,
        438.51448087216227
    )
    gui3d.camera.value.zoom = 1.2080332409972304
    gui3d.camera.value.quaternion.set(
        -0.23850469777524078,
        0.4732901060118559,
        0.13482987889740225,
        0.837217348391049
    )
    gui3d.camera.value.updateProjectionMatrix() // need to call after setting zoom
    gui3d.t_range.value.max = 19
    // make the lower part mostly transparent
    let fade_below = -1.1
    const fade_material = new THREE.MeshStandardMaterial({
        color: 0x000000,
        opacity: 0.05,
        transparent: true,
        side: THREE.FrontSide
    })
    for (let x of gui3d.gate_vec_meshes) {
        for (let y of x) {
            for (let z of y) {
                if (z == null) {
                    continue
                }
                for (let k of z) {
                    if (k.position.y < fade_below) {
                        k.material = fade_material
                    }
                }
            }
        }
    }
    for (let x of gui3d.noise_model_pauli_meshes) {
        for (let y of x) {
            for (let k of y) {
                if (k == null) {
                    continue
                }
                if (k.position.y < fade_below) {
                    gui3d.scene.remove(k)
                }
            }
        }
    }
    // change color to be consist with other plots
    gui3d.get_qubit_material("Data").color = new THREE.Color(0x000000)
    const color_x = new THREE.Color(0xEEFFEE)
    gui3d.get_qubit_material("StabX").color = color_x
    gui3d.get_gate_material("MeasureX").color = color_x
    gui3d.get_gate_material("InitializeX").color = color_x
    const color_z = new THREE.Color(0xCCDCF0)
    gui3d.get_qubit_material("StabZ").color = color_z
    gui3d.get_gate_material("MeasureZ").color = color_z
    gui3d.get_gate_material("InitializeZ").color = color_z
}
