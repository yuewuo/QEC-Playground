import * as gui3d from './gui3d.js'
import * as patches from './patches.js'

if (typeof window === 'undefined' || typeof document === 'undefined') {
    await import('./mocker.js')
}

window.gui3d = gui3d

const is_mock = typeof mockgl !== 'undefined'
if (is_mock) {
    global.gui3d = gui3d
    global.mocker = await import('./mocker.js')
}

// to work both in browser and nodejs
if (typeof Vue === 'undefined') {
    global.Vue = await import('vue')
}
const { ref, reactive, watch, computed } = Vue

// fetch visualization data
const urlParams = new URLSearchParams(window.location.search)
const filename = urlParams.get('filename') || "visualizer.json"

export var qecp_data
var patch_done = ref(false)

// alert(navigator.userAgent)
const is_chrome = navigator.userAgent.toLowerCase().indexOf('chrome/') > -1
const is_firefox = navigator.userAgent.toLowerCase().indexOf('firefox/') > -1
const is_browser_supported = ref(is_chrome || is_firefox)

export const case_select = ref(0)

// create vue3 app
const App = {
    setup() {
        return {
            error_message: ref(null),
            warning_message: ref(null),
            case_num: ref(1),
            case_select: case_select,
            case_select_label: ref(1),
            case_labels: ref([]),
            qecp_data_ready: ref(false),
            use_perspective_camera: gui3d.use_perspective_camera,
            sizes: gui3d.sizes,
            export_scale_selected: ref(1),
            export_resolution_options: ref([]),
            // GUI related states
            show_stats: gui3d.show_stats,
            show_config: gui3d.show_config,
            show_hover_effect: gui3d.show_hover_effect,
            lock_view: ref(false),
            is_browser_supported: is_browser_supported,
            // select
            current_selected: gui3d.current_selected,
            selected_vertex_neighbor_edges: ref([]),
            selected_vertex_attributes: ref(""),
            selected_edge: ref(null),
            selected_edge_attributes: ref(""),
            noise_model_info: ref(null),
            // display options
            display_qubits: gui3d.display_qubits,
            display_idle_sticks: gui3d.display_idle_sticks,
            display_gates: gui3d.display_gates,
            display_measurements: gui3d.display_measurements,
            display_error_pattern: gui3d.display_error_pattern,
            display_correction: gui3d.display_correction,
            existed_model_graph: gui3d.existed_model_graph,
            display_model_graph: gui3d.display_model_graph,
            model_graph_region_display: gui3d.model_graph_region_display,
            existed_model_hypergraph: gui3d.existed_model_hypergraph,
            display_model_hypergraph: gui3d.display_model_hypergraph,
            existed_noise_model: gui3d.existed_noise_model,
            display_noise_model_pauli: gui3d.display_noise_model_pauli,
            display_noise_model_erasure: gui3d.display_noise_model_erasure,
            t_range: gui3d.t_range,
            t_length: gui3d.t_length,
            non_zero_color: "red",
            zero_color: "grey",
        }
    },
    async mounted() {
        gui3d.root.style.setProperty('--control-visibility', 'visible')
        let response = null
        try {
            response = await fetch('./data/' + filename, { cache: 'no-cache', })
        } catch (e) {
            this.error_message = "fetch file error"
            throw e
        }
        if (response.ok || is_mock) {
            qecp_data = await response.json()
            // console.log(qecp_data)
            if (qecp_data.format != "qecp") {
                this.error_message = `visualization file format error, get "${qecp_data.format}" expected "qecp"`
                throw this.error_message
            }
        } else {
            this.error_message = `fetch file error ${response.status}: ${response.statusText}`
            throw this.error_message
        }
        // load case
        this.show_case(0)  // load the first case
        this.case_num = qecp_data.cases.length
        for (let idx=0; idx < qecp_data.cases.length; idx++) {
            let this_case = qecp_data.cases[idx]
            this.case_labels.push(`[${idx}]`)
        }
        this.case_select_label = this.case_labels[0]
        this.qecp_data_ready = true
        // only if data loads successfully will the animation starts
        if (!is_mock) {  // if mock, no need to refresh all the time
            gui3d.animate()
        }
        // add keyboard shortcuts
        document.onkeydown = (event) => {
            if (!event.metaKey) {
                if (event.key == "t" || event.key == "T") {
                    this.reset_camera("top")
                } else if (event.key == "l" || event.key == "L") {
                    this.reset_camera("left")
                } else if (event.key == "f" || event.key == "F") {
                    this.reset_camera("front")
                } else if (event.key == "c" || event.key == "C") {
                    this.show_config = !this.show_config
                } else if (event.key == "s" || event.key == "S") {
                    this.show_stats = !this.show_stats
                } else if (event.key == "v" || event.key == "V") {
                    this.lock_view = !this.lock_view
                } else if (event.key == "o" || event.key == "O") {
                    this.use_perspective_camera = false
                } else if (event.key == "p" || event.key == "P") {
                    this.use_perspective_camera = true
                } else if (event.key == "e" || event.key == "E") {
                    this.toggle_error_correction()
                } else if (event.key == "ArrowRight") {
                    if (this.case_select < this.case_num - 1) {
                        this.case_select += 1
                    }
                } else if (event.key == "ArrowLeft") {
                    if (this.case_select > 0) {
                        this.case_select -= 1
                    }
                } else if (event.key == "ArrowUp") {
                    if (this.t_range.max < this.t_length) {
                        this.t_range.max += 1
                    }
                    if (this.t_range.min < this.t_range.max - 1) {
                        this.t_range.min += 1
                    }
                } else if (event.key == "ArrowDown") {
                    if (this.t_range.min > 0) {
                        this.t_range.min -= 1
                    }
                    if (this.t_range.max > this.t_range.min + 1) {
                        this.t_range.max -= 1
                    }
                } else {
                    return  // unrecognized, propagate to other listeners
                }
                event.preventDefault()
                event.stopPropagation()
            }
        }
        // get command from url parameters
        await Vue.nextTick()
        let case_idx = urlParams.get('ci') || urlParams.get('case_idx')
        if (case_idx != null) {
            case_idx = parseInt(case_idx)
            if (case_idx < 0) {  // iterate from the end, like python list[-1]
                case_idx = this.case_num + case_idx
                if (case_idx < 0) {  // too small
                    case_idx = 0
                }
            }
            if (case_idx >= this.case_num) {
                case_idx = this.case_num - 1
            }
            this.case_select = case_idx
        }
        // update resolution options when sizes changed
        watch(gui3d.sizes, this.update_export_resolutions, { immediate: true })
        // execute patch scripts
        setTimeout(async () => {
            const patch_name = urlParams.get('patch')
            if (patch_name != null) {
                console.log(`running patch ${patch_name}`)
                const patch_function = patches[patch_name]
                await patch_function.bind(this)()
            }
            const patch_url = urlParams.get('patch_url')
            if (patch_url != null) {
                this.warning_message = `patching from external file: ${patch_url}`
                let patch_module = await import(patch_url)
                if (patch_module.patch == null) {
                    this.error_message = "invalid patch file: `patch` function not found"
                    throw "patch file error"
                }
                await patch_module.patch.bind(this)()
                this.warning_message = null
            }
            patch_done.value = true
        }, 100);
        await this.update_mathjax()
    },
    methods: {
        show_case(case_idx) {
            try {
                window.qecp_data = qecp_data
                window.case_idx = case_idx
                gui3d.show_case(case_idx, qecp_data)
            } catch (e) {
                this.error_message = "load data error"
                throw e
            }
        },
        reset_camera(direction) {
            gui3d.reset_camera_position(direction)
        },
        qubit_type_name(qubit_type) {
            if (qubit_type == "StabX") { return "X Ancilla" }
            if (qubit_type == "StabZ") { return "Z Ancilla" }
            if (qubit_type == "StabXZZXLogicalX") { return "XZZX Ancilla" }
            if (qubit_type == "StabXZZXLogicalZ") { return "XZZX Ancilla" }
            if (qubit_type == "StabY") { return "Y Ancilla" }
            return qubit_type
        },
        is_error_position(selected_type) {
            return selected_type == "idle_gate" || selected_type == "noise_model_node_pauli" || selected_type == "noise_model_node_erasure"
        },
        async update_selected_display() {
            this.noise_model_info = null
            const qecp_data = gui3d.active_qecp_data.value
            const case_idx = gui3d.active_case_idx.value
            if (this.current_selected == null || qecp_data == null || case_idx == null) return false
            if (this.is_error_position(this.current_selected.type)) {
                // compute information about the error position: including error rates, etc.
                const { t, i, j } = this.current_selected
                const node = qecp_data.simulator.nodes?.[t]?.[i]?.[j]
                const noise_model_node = qecp_data.noise_model.nodes?.[t]?.[i]?.[j]
                const contributed_vec = gui3d.contributed_noise_sources?.[t]?.[i]?.[j]
                if (node != null && noise_model_node != null && contributed_vec != null) {
                    let noise_model_info = {}
                    Object.assign(noise_model_info, noise_model_node)
                    noise_model_info.pp.sum = noise_model_info.pp.px + noise_model_info.pp.py + noise_model_info.pp.pz
                    if (noise_model_info.corr_pe != null) {
                        noise_model_info.corr_pe.sum = noise_model_info.corr_pe.pie + noise_model_info.corr_pe.pei + noise_model_info.corr_pe.pee
                    }
                    if (noise_model_info.corr_pp != null) {
                        noise_model_info.corr_pp.sum = noise_model_info.corr_pp.pix + noise_model_info.corr_pp.piy + noise_model_info.corr_pp.piz
                            + noise_model_info.corr_pp.pxi + noise_model_info.corr_pp.pxx + noise_model_info.corr_pp.pxy + noise_model_info.corr_pp.pxz
                            + noise_model_info.corr_pp.pyi + noise_model_info.corr_pp.pyx + noise_model_info.corr_pp.pyy + noise_model_info.corr_pp.pyz
                            + noise_model_info.corr_pp.pzi + noise_model_info.corr_pp.pzx + noise_model_info.corr_pp.pzy + noise_model_info.corr_pp.pzz
                    }
                    // add contributed noise sources
                    noise_model_info.additional = []
                    for (const contributed of contributed_vec) {
                        if (contributed.others != true) continue
                        const ad = {}
                        Object.assign(ad, contributed)
                        if (ad.add_idx != null) {
                            const noise = qecp_data.noise_model.additional_noise[ad.add_idx]
                            ad.noise = noise
                        }
                        noise_model_info.additional.push(ad)
                    }

                    this.noise_model_info = noise_model_info
                }
                await Vue.nextTick()
                await this.update_mathjax()
            }
        },
        async update_mathjax() {
            for (let i=0; i<100; ++i) await Vue.nextTick()
            await MathJax.typesetPromise()
        },
        build_data_pos(pos_str) {
            const data = gui3d.get_position(pos_str)
            const qecp_data = gui3d.active_qecp_data.value
            const node = qecp_data.simulator.nodes?.[data.t]?.[data.i]?.[data.j]
            if (node != null) {
                data.gate_peer = node.gp
            }
            return data
        },
        async ref_btn_hover(pos_str) {
            await this.jump_to("idle_gate", this.build_data_pos(pos_str), false)
        },
        async ref_btn_leave(pos_str) {
            await this.jump_to(null, null, false)
        },
        async ref_btn_click(pos_str) {
            await this.jump_to("idle_gate", this.build_data_pos(pos_str), true)
        },
        async jump_to(type, data, is_click=true) {
            let current_ref = is_click ? gui3d.current_selected : gui3d.current_hover
            await Vue.nextTick()
            if (type == null) {
                current_ref.value = null
                await Vue.nextTick()
            } else {
                data.type = type
                current_ref.value = data
                await Vue.nextTick()
            }
        },
        update_export_resolutions() {
            this.export_resolution_options.splice(0, this.export_resolution_options.length)
            let exists_in_new_resolution = false
            for (let i=-100; i<100; ++i) {
                let scale = 1 * Math.pow(10, i/10)
                let width = Math.round(this.sizes.canvas_width * scale)
                let height = Math.round(this.sizes.canvas_height * scale)
                if (width > 5000 || height > 5000) {  // to large, likely exceeds WebGL maximum buffer size
                    break
                }
                if (width >= 300 || height >= 300) {
                    let label = `${width} x ${height}`
                    this.export_resolution_options.push({
                        label: label,
                        value: scale,
                    })
                    if (scale == this.export_scale_selected) {
                        exists_in_new_resolution = true
                    }
                }
            }
            if (!exists_in_new_resolution) {
                this.export_scale_selected = null
            }
        },
        preview_image() {
            const data = gui3d.render_png(this.export_scale_selected)
            gui3d.open_png(data)
        },
        download_image() {
            const data = gui3d.render_png(this.export_scale_selected)
            gui3d.download_png(data)
        },
        get_idx_from_label(label) {
            return parseInt(label.split(']')[0].split('[')[1])
        },
        get_case(case_idx) {
            return qecp_data.cases[case_idx]
        },
        toggle_error_correction() {
            if (this.display_error_pattern) {
                this.display_error_pattern = false
                this.display_correction = true
            } else {
                this.display_error_pattern = true
                this.display_correction = false
            }
        },
    },
    watch: {
        async case_select() {
            // console.log(this.case_select)
            this.show_case(this.case_select)  // load the case
            this.case_select_label = this.case_labels[this.case_select]
            for (const _ of Array(4).keys()) await Vue.nextTick()
            await this.update_selected_display()
        },
        case_select_label() {
            this.case_select = this.get_idx_from_label(this.case_select_label)
        },
        async current_selected() {
            await this.update_selected_display()
        },
        lock_view() {
            gui3d.enable_control.value = !this.lock_view
        },
    },
    computed: {
        scale() {
            return this.sizes.scale
        },
        vertical_thumb_style() {
            return {
                right: `4px`,
                borderRadius: `5px`,
                backgroundColor: '#027be3',
                width: `5px`,
                opacity: 0.75
            }
        },
        horizontal_thumb_style() {
            return {
                bottom: `4px`,
                borderRadius: `5px`,
                backgroundColor: '#027be3',
                height: `5px`,
                opacity: 0.75
            }
        },
        vertical_bar_style() {
            return {
                right: `2px`,
                borderRadius: `9px`,
                backgroundColor: '#027be3',
                width: `9px`,
                opacity: 0.2
            }
        },
        horizontal_bar_style() {
            return {
                bottom: `2px`,
                borderRadius: `9px`,
                backgroundColor: '#027be3',
                height: `9px`,
                opacity: 0.2
            }
        },
        active_case() {
            if (!this.qecp_data_ready) return
            return qecp_data.cases[this.case_select]
        },
        model_graph_region_options() {
            let regions = gui3d.model_graph_regions.value
            let options = []
            for (let i=0; i<regions; ++i) {
                options.push({ label: (i==0?"regions: ":"") + `${i}`, value: i, color: gui3d.sequential_colors[i][0] })
            }
            return options
        },
    },
}

if (!is_mock) {
    const app = Vue.createApp(App)
    app.use(Quasar)
    window.app = app.mount("#app")
} else {
    global.Element = window.Element
    global.SVGElement = window.SVGElement  // https://github.com/jsdom/jsdom/issues/2734
    App.template = "<div></div>"
    const app = Vue.createApp(App)
    window.app = app.mount("#app")
    while (!patch_done.value) {
        await sleep(50)
    }
    for (let i=0; i<10; ++i) {
        await sleep(10)
        await Vue.nextTick()
    }
    console.log("[rendering]")
    const pixels = await gui3d.nodejs_render_png()
    console.log("[saving]")
    mocker.save_pixels(pixels)
}
