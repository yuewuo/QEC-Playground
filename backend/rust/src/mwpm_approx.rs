use super::ndarray;

pub fn minimum_weight_perfect_matching_approx(node_count: usize, weighted_edges: Vec<(usize, usize, f64)>, substreams: usize) -> Vec<(usize, usize, f64)> {
    let mut cardinal_matchings = vec![Vec::<(usize, usize, f64)>::new(); substreams];
    let mut matching_bits = ndarray::Array::from_elem((node_count,substreams), false);
    let mut substream_weights = Vec::<f64>::new();

    // ε=L√wmax − 1
    // let wmax = 3.0*(distance as f64); //Is this correct?
    // let epsilon_plus_1 = wmax.abs().powf(1.0/substreams as f64);
    // for i in 0..substreams {
        // substream_weights.push(epsilon_plus_1.powf(i as f64));
    // }
    
    // We simplify the weights by approximating the above as follows
    // SS 0 : weight 1, SS 1 : weight 2, SS last : weights above last + 1

    for i in 0..substreams {
        substream_weights.push(i as f64 + 1.0);
    }

    for (u,v,w) in weighted_edges.iter() {
        let mut has_added = false;
        for i in 0..substreams {
            if *w  <= substream_weights[i] {
                if !matching_bits[[*u,i]] && !matching_bits[[*v,i]] {
                    matching_bits[[*u,i]] = true;
                    matching_bits[[*v,i]] = true;
                    if !has_added {//Add e only  once to the  matchings
                        cardinal_matchings[i].push((*u,*v,*w)); 
                        has_added =true;
                    }
                } 
            }
        }
    }

    let mut tbits = ndarray::Array::from_elem(node_count, false);
    let mut maximum_matching = Vec::<(usize, usize, f64)>::new();

    for i in 0..substreams {
        for (u,v,w) in cardinal_matchings[i].iter() {
            if !tbits[[*u]] && !tbits[[*v]] {
                tbits[[*u]] = true;
                tbits[[*v]] = true;
                // T.push((*u,*v,*w));
                maximum_matching.push((*u,*v,*w));
            }
        }
    }
    // println!("{:?}",maximum_matching);
    maximum_matching
}

// In here we do a modified matching ignoring the boundary and then later check for the boundary conditions
pub fn minimum_weight_perfect_matching_approx_modified(node_count: usize, weighted_edges: Vec<(usize, usize, f64)>, substreams: usize) -> Vec<(usize, usize, f64)> {
    // let syndrome_count_is_even = node_count%4 == 0; // since node count also contain virtual edges actual syndrome count is half node count
    let mut boundry_weight = ndarray::Array::from_elem(node_count/2, (0,0.0));
    let mut node_used = ndarray::Array::from_elem(node_count/2, false);
    let mut weighted_edges_non_boundry = Vec::<(usize, usize, f64)>::new();

    for (u,v,w) in weighted_edges.iter() {
        if *u >= node_count/2 {
            boundry_weight[*v] = (*u,*w);
        } else if *v >= node_count/2 {
            boundry_weight[*u] = (*v,*w);
        } else {
            weighted_edges_non_boundry.push((*u,*v,*w));
        }
    }

    let initial_matching = minimum_weight_perfect_matching_approx(node_count/2, weighted_edges_non_boundry, substreams);
    let mut final_matching = Vec::<(usize, usize,f64)>::new();

    for (u,v,w) in initial_matching.iter(){
        if boundry_weight[[*u]].1 + boundry_weight[[*v]].1 < *w {
            final_matching.push((*u,boundry_weight[[*u]].0,boundry_weight[[*u]].1));
            final_matching.push((*v,boundry_weight[[*u]].0,boundry_weight[[*v]].1));
        } else {
            final_matching.push((*u,*v,*w));
        }
        node_used[*u] = true;
        node_used[*v] = true;
    }

    for i in 0..node_count/2 {
        if node_used[i] == false {
            final_matching.push((i,boundry_weight[i].0,boundry_weight[i].1));
        }
    }


    final_matching
}