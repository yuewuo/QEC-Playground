#![allow(non_snake_case)]

use super::types::*;

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
    let z_correction = corrections[0].clone();
    let x_correction = corrections[1].clone();
    (x_correction, z_correction)
}
