use super::ndarray;

pub fn minimum_weight_perfect_matching_approx(node_num: usize, weighted_edges: Vec<(usize, usize, f64)>, distance: usize, L: usize) -> Vec<usize> {
    let mut C = vec![Vec::<(usize, usize, f64)>::new(); L];
    let mut MB = ndarray::Array::from_elem((L,L), false);
    let mut substream_weights = Vec::<f64>::new();
    // ε=L√wmax − 1
    let wmax = 3.0*(distance as f64); //Is this correct?
    let epsilon_plus_1 = wmax.abs().powf(1.0/L as f64);
    for i in 0..L {
        substream_weights.push(epsilon_plus_1.powf(i as f64));
    }
    let mut has_added = false;
    for (u,v,w) in weighted_edges.iter() {
        has_added = false;
        for i in (0..L).rev() {
            if w  >= &substream_weights[i] {
                if !MB[[*u,i]] && !MB[[*v,i]] {
                    MB[[*u,i]] = true;
                    MB[[*v,i]] = true;
                    if !has_added {//Add e only  once to the  matchings
                        C[i].push((*u,*v,*w)); 
                        has_added =true;
                    }
                } 
            }
        }
    }

    let mut tbits = ndarray::Array::from_elem((L), false);
    let mut T = Vec::<usize>::new();

    for i in (0..L).rev() {
        for (u,v,w) in C[i].iter() {
            if !tbits[[*u]] && !tbits[[*v]] {
                tbits[[*u]] = true;
                tbits[[*v]] = true;
                // T.push((*u,*v,*w));
                T.push(*u);
            }
        }
    }

    T
}
