#![allow(non_snake_case)]

use super::types::*;
use super::blossom;

/// This is a STUPID error correction algorithm !!!
/// It's even not fault tolerant at all. It just make sure all the stabilizers are back to +1 eigenstate
/// return `(x_correction, z_correction)`
pub fn stupid_correction(measurement: &ZxMeasurement) -> (ZxCorrection, ZxCorrection) {
    #[derive(Debug, Clone)]
    enum Connection {
        Boundary,
        ErrorPoint(usize),
    }
    #[derive(Debug, Clone)]
    struct ErrorPoint {
        at: usize,  // the index of Vertex
        connected_to: Option<Connection>,
    }
    #[derive(Debug, Clone)]
    struct Vertex {
        i: usize,
        j: usize,
        connections: Vec<usize>,  // indexes of other vertices
        propagated_error: Option<usize>,
        propagated_from: Option<usize>,
        connect_to_boundary: Option<(usize, usize)>,   // if connect to boundary, that would be the boundary data qubit axis
    }
    let L = measurement.L();
    let mut corrections = Vec::<ZxCorrection>::new();
    for is_z_stabilizer in [false, true].iter() {
        let is_z_stabilizer = *is_z_stabilizer;
        let mut correction = ZxCorrection::new_L(L);
        // build the graph of vertices and error points first
        let mut vertices = Vec::<Vertex>::new();
        let mut error_points = Vec::<ErrorPoint>::new();
        let mut pos_to_index_ro = ndarray::Array::from_elem((L+1, L+1), -1isize);
        let mut pos_to_index = pos_to_index_ro.view_mut();
        for i in 0..L+1 {
            for j in 0..L+1 {
                if is_z_stabilizer && j != 0 && j != L && (i + j) % 2 == 0 {  // Z stabilizer only when i+j is even
                    let mut vertex = Vertex {
                        i: i,
                        j: j,
                        connections: Vec::<usize>::new(),
                        propagated_error: None,
                        propagated_from: None,
                        connect_to_boundary: None,
                    };
                    if j == 1 {  // boundary is on left
                        vertex.connect_to_boundary = Some((i-1, 0));
                    }
                    if j == L - 1 {  // boundary is on right
                        vertex.connect_to_boundary = Some((i, L-1));
                    }
                    pos_to_index[[i, j]] = vertices.len() as isize;
                    if measurement[[i, j]] {
                        let error_point = ErrorPoint {
                            at: vertices.len(),
                            connected_to: None,
                        };
                        vertex.propagated_error = Some(error_points.len());  // mark it as having error
                        error_points.push(error_point);
                    }
                    vertices.push(vertex);
                }
                if !is_z_stabilizer && i != 0 && i != L && (i + j) % 2 == 1 {  // X stabilizer only when i+j is odd
                    let mut vertex = Vertex {
                        i: i,
                        j: j,
                        connections: Vec::<usize>::new(),
                        propagated_error: None,
                        propagated_from: None,
                        connect_to_boundary: None,
                    };
                    if i == 1 {  // boundary is on top
                        vertex.connect_to_boundary = Some((0, j));
                    }
                    if i == L - 1 {  // boundary is on right
                        vertex.connect_to_boundary = Some((L-1, j-1));
                    }
                    pos_to_index[[i, j]] = vertices.len() as isize;
                    if measurement[[i, j]] {
                        let error_point = ErrorPoint {
                            at: vertices.len(),
                            connected_to: None,
                        };
                        vertex.propagated_error = Some(error_points.len());  // mark it as having error
                        error_points.push(error_point);
                    }
                    vertices.push(vertex);
                }
            }
        }
        // calculate Vertex.connections
        for vertex in vertices.iter_mut() {
            let i = vertex.i as isize;
            let j = vertex.j as isize;
            for ip in [i-1, i+1].iter() {
                for jp in [j-1, j+1].iter() {
                    let ip = *ip;
                    let jp = *jp;
                    if ip >=0 && ip as usize <= L && jp >= 0 && jp as usize <= L && pos_to_index[[ip as usize, jp as usize]] >= 0 {
                        vertex.connections.push(pos_to_index[[ip as usize, jp as usize]] as usize);
                    }
                }
            }
        }
        // for (index, vertex) in vertices.iter_mut().enumerate() {
        //     println!("{}: {:?}", index, vertex);
        // }
        // for (index, error_point) in error_points.iter_mut().enumerate() {
        //     println!("{}: {:?}", index, error_point);
        // }
        let mut something_changed = true;
        // propagate every error
        while something_changed {
            something_changed = false;
            for index in 0..error_points.len() {
                if error_points[index].connected_to.is_some() {
                    continue  // stop propagating those matched ones
                }
                let mut to_be_propagated = Vec::<(usize, usize)>::new();  // the index of element and the index to be propagated
                let mut boundary_v_idx: Option<(usize, usize, usize)> = None;  // if exists, (`v_idx`, `boundary_i`, `boundary_j`)
                for (v_idx, vertex) in vertices.iter().enumerate() {
                    if vertex.propagated_error == Some(index) {
                        for connection in vertex.connections.iter() {
                            let connection = *connection;
                            to_be_propagated.push((v_idx, connection));
                        }
                        if let Some((boundary_i, boundary_j)) = vertex.connect_to_boundary {
                            boundary_v_idx = Some((v_idx, boundary_i, boundary_j));
                        }
                    }
                }
                // first see if it propagates to other error points that hasn't been connected yet
                for (last_idx, v_idx) in to_be_propagated.iter() {
                    let v_idx = *v_idx;
                    let last_idx = *last_idx;
                    if let Some(propagated_error) = vertices[v_idx].propagated_error {
                        // only care if it's not propagating back
                        if propagated_error != index {
                            // if the propagated target is not yet matched, then match them
                            if error_points[propagated_error].connected_to.is_none() {
                                // mark both error points as connected
                                error_points[propagated_error].connected_to = Some(Connection::ErrorPoint(index));
                                error_points[index].connected_to = Some(Connection::ErrorPoint(propagated_error));
                                // change the correction matrix along this path
                                let mut idx = last_idx;
                                let stop_error_idx = error_points[index].at;
                                while idx != stop_error_idx {
                                    let i1 = vertices[idx].i;
                                    let j1 = vertices[idx].j;
                                    let back_idx = vertices[idx].propagated_from.expect("must have set `propagated_from`");  // must have `propagated_from`
                                    let i2 = vertices[back_idx].i;
                                    let j2 = vertices[back_idx].j;
                                    let i = (i1 + i2 - 1) / 2;
                                    let j = (j1 + j2 - 1) / 2;
                                    correction[[i, j]] ^= true;
                                    idx = back_idx;
                                }
                                let stop_error_idx = error_points[propagated_error].at;
                                idx = v_idx;
                                while idx != stop_error_idx {
                                    let i1 = vertices[idx].i;
                                    let j1 = vertices[idx].j;
                                    let back_idx = vertices[idx].propagated_from.expect("must have set `propagated_from`");  // must have `propagated_from`
                                    let i2 = vertices[back_idx].i;
                                    let j2 = vertices[back_idx].j;
                                    let i = (i1 + i2 - 1) / 2;
                                    let j = (j1 + j2 - 1) / 2;
                                    correction[[i, j]] ^= true;
                                    idx = back_idx;
                                }
                                let i1 = vertices[v_idx].i;
                                let j1 = vertices[v_idx].j;
                                let i2 = vertices[last_idx].i;
                                let j2 = vertices[last_idx].j;
                                let i = (i1 + i2 - 1) / 2;
                                let j = (j1 + j2 - 1) / 2;
                                correction[[i, j]] ^= true;
                                break  // stop propagating because it already matches with others
                            } else {  // also propagate here
                                something_changed = true;
                                vertices[v_idx].propagated_from = Some(last_idx);
                                vertices[v_idx].propagated_error = Some(index);
                            }
                        }
                    } else {  // propagate here
                        something_changed = true;
                        vertices[v_idx].propagated_from = Some(last_idx);
                        vertices[v_idx].propagated_error = Some(index);
                    }
                }
                if error_points[index].connected_to.is_some() {
                    continue  // stop propagating those matched ones
                }
                // then see if it propagates to boundary    
                if let Some((v_idx, boundary_i, boundary_j)) = boundary_v_idx {  // connected to boundary through `v_idx`
                    error_points[index].connected_to = Some(Connection::Boundary);
                    // change the correction matrix along this path
                    correction[[boundary_i, boundary_j]] ^= true;
                    let mut idx = v_idx;
                    let stop_error_idx = error_points[index].at;
                    while idx != stop_error_idx {
                        let i1 = vertices[idx].i;
                        let j1 = vertices[idx].j;
                        let back_idx = vertices[idx].propagated_from.expect("must have set `propagated_from`");  // must have `propagated_from`
                        let i2 = vertices[back_idx].i;
                        let j2 = vertices[back_idx].j;
                        let i = (i1 + i2 - 1) / 2;
                        let j = (j1 + j2 - 1) / 2;
                        correction[[i, j]] ^= true;
                        idx = back_idx;
                    }
                }
            }
        }
        corrections.push(correction);
    }
    let x_correction = corrections[1].clone();
    let z_correction = corrections[0].clone();
    (x_correction, z_correction)
}

/// Try to use blossom graph library to decode
/// return `(x_correction, z_correction)`
pub fn try_blossom_correction(measurement: &ZxMeasurement) -> (ZxCorrection, ZxCorrection) {
    let L = measurement.L();
    let mut corrections = Vec::<ZxCorrection>::new();
    for is_z_stabilizer in [false, true].iter() {
        let is_z_stabilizer = *is_z_stabilizer;
        let correction = ZxCorrection::new_L(L);
        let mut error_vertices = Vec::<(usize, usize, usize)>::new();  // (i, j, boundary)
        for i in 0..L+1 {
            for j in 0..L+1 {
                if is_z_stabilizer && j != 0 && j != L && (i + j) % 2 == 0 && measurement[[i, j]] {  // Z stabilizer only when i+j is even
                    // boundary is on left and right
                    error_vertices.push((i, j, std::cmp::min(j, L - j)))
                }
                if !is_z_stabilizer && i != 0 && i != L && (i + j) % 2 == 1 && measurement[[i, j]] {  // X stabilizer only when i+j is odd
                    // boundary is on top and bottom
                    error_vertices.push((i, j, std::cmp::min(i, L - i)))
                }
            }
        }
        let error_counts = error_vertices.len();
        if error_counts != 0 {  // only when some error occurs
            let distance_delta = |i: isize, j: isize| ((i+j).abs() + (i-j).abs()) / 2;
            let weight_of = |i1: usize, j1: usize, i2: usize, j2: usize| - distance_delta((i2 as isize) - (i1 as isize), (j2 as isize) - (j1 as isize));
            let mut graph_vertices = std::collections::HashMap::new();
            for i in 0..error_counts {
                let (i1, j1, boundary) = error_vertices[i];
                let boundary_vertex_idx = i + error_counts;
                let mut vertices = vec![boundary_vertex_idx];
                let mut weights = vec![-(boundary as blossom::weighted::Weight)];
                for j in 0..error_counts {
                    if j != i {
                        let (i2, j2, _) = error_vertices[j];
                        weights.push(weight_of(i1, j1, i2, j2) as blossom::weighted::Weight);
                        vertices.push(j);
                    }
                }
                graph_vertices.insert(i, (vertices, weights));
                // add boundary connections of `boundary_vertex_idx`
                let mut boundary_vertices = vec![i];
                let mut boundary_weights = vec![-(boundary as blossom::weighted::Weight)];
                for j in 0..error_counts {
                    if j != i {
                        boundary_weights.push(0 as blossom::weighted::Weight);
                        boundary_vertices.push(j + error_counts);
                    }
                }
                graph_vertices.insert(boundary_vertex_idx, (boundary_vertices, boundary_weights));
            }
            println!("{:?}", graph_vertices);
            let graph = blossom::weighted::WeightedGraph::new(graph_vertices);
            let matching = graph.maximin_matching().unwrap();
            println!("matching: {:?}", matching);
            let matching_edges = matching.edges();
            println!("matching_edges: {:?}", matching_edges);

            // FAILED: `graph.maximin_matching` does not work as expected.
            // with input {3: ([0, 4, 5], [-1.0, 0.0, 0.0]), 2: ([5, 0, 1], [-1.0, -3.0, -2.0]), 4: ([1, 3, 5], [-2.0, 0.0, 0.0]), 0: ([3, 1, 2], [-1.0, -3.0, -3.0]), 1: ([4, 0, 2], [-2.0, -3.0, -2.0]), 5: ([2, 3, 4], [-1.0, 0.0, 0.0])}
            // it matches [(2, 5), (0, 3), (1, 4)]
            // this is absolutely not optimal. It seems to find a local maximal and stop there.

        }
        corrections.push(correction);
    }
    let x_correction = corrections[1].clone();
    let z_correction = corrections[0].clone();
    (x_correction, z_correction)
}

/// Try to use some `maximum_max_weight_matching` function to decode
/// return `(x_correction, z_correction)`
pub fn maximum_max_weight_matching_correction<F>(measurement: &ZxMeasurement, maximum_max_weight_matching: F) -> (ZxCorrection, ZxCorrection)
        where F: Fn(Vec<(usize, usize, f64)>) -> std::collections::HashSet<(usize, usize)> {
    let distance_delta = |i: isize, j: isize| ((i+j).abs() + (i-j).abs()) / 2;
    let distance = |i1: isize, j1: isize, i2: isize, j2: isize| distance_delta(i2 - i1, j2 - j1);
    let weight_of = |i1: usize, j1: usize, i2: usize, j2: usize| - distance(i1 as isize, j1 as isize, i2 as isize, j2 as isize) as f64;
    return maximum_max_weight_matching_correction_weighted(measurement, maximum_max_weight_matching, weight_of);
}
pub fn maximum_max_weight_matching_correction_weighted<F, F2>(measurement: &ZxMeasurement, maximum_max_weight_matching: F, weight_of: F2) -> (ZxCorrection, ZxCorrection)
        where F: Fn(Vec<(usize, usize, f64)>) -> std::collections::HashSet<(usize, usize)>, 
        F2: Fn(usize, usize, usize, usize) -> f64 {
    let L = measurement.L();
    let mut corrections = Vec::<ZxCorrection>::new();
    for is_z_stabilizer in [false, true].iter() {
        let is_z_stabilizer = *is_z_stabilizer;
        let mut correction = ZxCorrection::new_L(L);
        let mut error_vertices = Vec::<(usize, usize, usize)>::new();  // (i, j, boundary)
        let is_z_stab_i_j = |i, j| j != 0 && j != L && (i + j) % 2 == 0;
        let is_x_stab_i_j = |i, j| i != 0 && i != L && (i + j) % 2 == 1;
        for i in 0..L+1 {
            for j in 0..L+1 {
                if is_z_stabilizer && is_z_stab_i_j(i, j) && measurement[[i, j]] {  // Z stabilizer only when i+j is even
                    // boundary is on left and right
                    error_vertices.push((i, j, std::cmp::min(j, L - j)))
                }
                if !is_z_stabilizer && is_x_stab_i_j(i, j) && measurement[[i, j]] {  // X stabilizer only when i+j is odd
                    // boundary is on top and bottom
                    error_vertices.push((i, j, std::cmp::min(i, L - i)))
                }
            }
        }
        let error_counts = error_vertices.len();
        if error_counts != 0 {  // only when some error occurs
            let distance_delta = |i: isize, j: isize| ((i+j).abs() + (i-j).abs()) / 2;
            let distance = |i1: isize, j1: isize, i2: isize, j2: isize| distance_delta(i2 - i1, j2 - j1);
            let mut edges = Vec::new();
            for a in 0..error_counts {
                let (i1, j1, boundary) = error_vertices[a];
                if a+1 < error_counts {  // add edges to other vertices
                    for b in a+1..error_counts {
                        let (i2, j2, _) = error_vertices[b];
                        edges.push((a, b, weight_of(i1, j1, i2, j2) as f64));
                    }
                }
                // add boundary connection to this
                edges.push((a, a + error_counts, - (boundary as f64)));
                // add boundary connections of `boundary_vertex_idx`
                if a+1 < error_counts {
                    for b in a+1..error_counts {
                        edges.push((a + error_counts, b + error_counts, 0 as f64));
                    }
                }
            }
            let matched = maximum_max_weight_matching(edges);
            let is_boundary = |a| a >= error_counts;
            for (a, b) in &matched {
                let a = *a;
                let b = *b;
                let connect_qubits = |a, b, correction: &mut ZxCorrection| {
                    let (mut i1, mut j1, _) = error_vertices[a];  // moving (i1,j1) -> (i2,j2)
                    let (i2, j2, _) = error_vertices[b];
                    while i1 != i2 || j1 != j2 {
                        let dist = distance(i1 as isize, j1 as isize, i2 as isize, j2 as isize);
                        for (delta_i, delta_j) in [(-1,-1), (-1,1), (1,-1), (1,1)].iter() {
                            let i1n = i1 as isize + *delta_i;
                            let j1n = j1 as isize + *delta_j;
                            let is_valid_stab_i_j = |i, j| i >= 0 && j >= 0 && i <= L as isize && j <= L as isize && (
                                if is_z_stabilizer { is_z_stab_i_j(i as usize, j as usize) } else { is_x_stab_i_j(i as usize, j as usize) });
                            if is_valid_stab_i_j(i1n, j1n) && distance(i1n, j1n, i2 as isize, j2 as isize) < dist {
                                // println!("from ({},{}) to ({},{})", i1, j1, i1n, j1n);
                                let i = (i1 + i1n as usize - 1) / 2;
                                let j = (j1 + j1n as usize - 1) / 2;
                                correction[[i, j]] ^= true;  // correct the data qubit on this path
                                i1 = i1n as usize;
                                j1 = j1n as usize;
                                break
                            }
                        }
                    }
                };
                let connect_boundary = |a, correction: &mut ZxCorrection| {
                    let (mut i1, mut j1, _) = error_vertices[a];  // moving (i1,j1) -> boundary
                    // if `is_z_stabilizer` then boundary is on left and right (+-j) otherwise the boundary is on top and bottom
                    let is_boundary_i_j = |i, j| if is_z_stabilizer { j == 0 || j == L } else { i == 0 || i == L };
                    while !is_boundary_i_j(i1, j1) {
                        let delta = if is_z_stabilizer {
                            if j1 < L - j1 {
                                [(-1, -1), (1, -1)]
                            } else {
                                [(-1, 1), (1, 1)]
                            }
                        } else {
                            if i1 < L - i1 {
                                [(-1, -1), (-1, 1)]
                            } else {
                                [(1, -1), (1, 1)]
                            }
                        };
                        for (delta_i, delta_j) in delta.iter() {
                            let i1n = i1 as isize + *delta_i;
                            let j1n = j1 as isize + *delta_j;
                            let is_valid_i_j = |i, j| i >= 0 && j >= 0 && i <= L as isize && j <= L as isize;
                            if is_valid_i_j(i1n, i1n) {
                                // println!("from ({},{}) to ({},{})", i1, j1, i1n, j1n);
                                let i = (i1 + i1n as usize - 1) / 2;
                                let j = (j1 + j1n as usize - 1) / 2;
                                correction[[i, j]] ^= true;  // correct the data qubit on this path
                                i1 = i1n as usize;
                                j1 = j1n as usize;
                                break
                            }
                        }
                    }
                };
                match (is_boundary(a), is_boundary(b)) {
                    (false, false) => { connect_qubits(a, b, &mut correction); }
                    (false, true) => { connect_boundary(a, &mut correction); }
                    (true, false) => { connect_boundary(b, &mut correction); }
                    _ => { }
                }
            }
        }
        corrections.push(correction);
    }
    let x_correction = corrections[1].clone();
    let z_correction = corrections[0].clone();
    (x_correction, z_correction)
}
