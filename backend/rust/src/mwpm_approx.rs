use super::ndarray;

pub fn minimum_weight_perfect_matching_approx(node_count: usize, weighted_edges: Vec<(usize, usize, f64)>, substreams: usize) -> Vec<(usize, usize)> {
    let mut cardinal_matchings = vec![Vec::<(usize, usize)>::new(); substreams];
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
            if w  <= &substream_weights[i] {
                if !matching_bits[[*u,i]] && !matching_bits[[*v,i]] {
                    matching_bits[[*u,i]] = true;
                    matching_bits[[*v,i]] = true;
                    if !has_added {//Add e only  once to the  matchings
                        cardinal_matchings[i].push((*u,*v)); 
                        has_added =true;
                    }
                } 
            }
        }
    }

    let mut tbits = ndarray::Array::from_elem(substreams, false);
    let mut maximum_matching = Vec::<(usize, usize)>::new();

    for i in 0..substreams {
        for (u,v) in cardinal_matchings[i].iter() {
            if !tbits[[*u]] && !tbits[[*v]] {
                tbits[[*u]] = true;
                tbits[[*v]] = true;
                // T.push((*u,*v,*w));
                maximum_matching.push((*u,*v));
            }
        }
    }

    maximum_matching
}
