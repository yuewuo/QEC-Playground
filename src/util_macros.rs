
/// instead of using iterator that generates a new `Position` each iteration, here I use macro to generate more efficient code
/// (increased 10% performance boost)
#[macro_export]
macro_rules! simulator_iter_loop {
    ($simulator:ident, $position:ident, $node:ident, $filter:expr, $body:expr, $start_t:expr, $end_t:expr, $delta_t:expr, $node_getter:expr) => {
        if $simulator.height != 0 && $simulator.vertical != 0 && $simulator.horizontal != 0 {
            let mut $position = Position::new($start_t, 0, 0);
            loop {
                {  // immutable scope
                    let $position = &$position;
                    if $filter {
                        for __simulator_iter_loop_internal_variable in 0..1 {
                            let $node = $node_getter;
                            $body
                        }
                    }
                }
                $position.j += 1;
                if $position.j >= $simulator.horizontal {
                    $position.j = 0;
                    $position.i += 1;
                    if $position.i >= $simulator.vertical {
                        $position.i = 0;
                        $position.t += $delta_t;
                        if $position.t >= $end_t {  // invalid position, stop here
                            break
                        }
                    }
                }
            }
        }
    };
}
#[allow(unused_imports)] pub use simulator_iter_loop;

#[macro_export]
macro_rules! simulator_iter_with_filter {
    ($simulator:ident, $position:ident, $filter:expr, $body:expr) => {
        simulator_iter_loop!($simulator, $position, __unused_node, $filter, $body, 0, $simulator.height, 1, Option::<bool>::None)
    };
    ($simulator:ident, $position:ident, $filter:expr, delta_t => $delta_t:expr, $body:expr) => {
        simulator_iter_loop!($simulator, $position, __unused_node, $filter, $body, 0, $simulator.height, $delta_t, Option::<bool>::None)
    };
    ($simulator:ident, $position:ident, $node:ident, $filter:expr, delta_t => $delta_t:expr, $body:expr) => {
        simulator_iter_loop!($simulator, $position, $node, $filter, $body, 0, $simulator.height, $delta_t, $simulator.get_node_unwrap($position))
    };
    ($simulator:ident, $position:ident, $node:ident, $filter:expr, $body:expr) => {
        simulator_iter_loop!($simulator, $position, $node, $filter, $body, 0, $simulator.height, 1, $simulator.get_node_unwrap($position))
    };
    ($simulator:ident, $position:ident, $node:ident, $filter:expr, t => $t:expr, $body:expr) => {
        simulator_iter_loop!($simulator, $position, $node, $filter, $body, $t, $t+1, 1, $simulator.get_node_unwrap($position))
    };
}
#[allow(unused_imports)] pub use simulator_iter_with_filter;

#[macro_export]
macro_rules! simulator_iter {
    ($simulator:ident, $position:ident, $body:expr) => {
        simulator_iter_with_filter!($simulator, $position, $simulator.is_node_exist(&$position), $body)
    };
    ($simulator:ident, $position:ident, delta_t => $delta_t:expr, $body:expr) => {
        simulator_iter_with_filter!($simulator, $position, $simulator.is_node_exist(&$position), delta_t => $delta_t, $body)
    };
    ($simulator:ident, $position:ident, $node:ident, delta_t => $delta_t:expr, $body:expr) => {
        simulator_iter_with_filter!($simulator, $position, $node, $simulator.is_node_exist(&$position), delta_t => $delta_t, $body)
    };
    ($simulator:ident, $position:ident, $node:ident, $body:expr) => {
        simulator_iter_with_filter!($simulator, $position, $node, $simulator.is_node_exist(&$position), $body)
    };
    ($simulator:ident, $position:ident, $node:ident, t => $t:expr, $body:expr) => {
        simulator_iter_with_filter!($simulator, $position, $node, $simulator.is_node_exist(&$position), t => $t, $body)
    };
}
#[allow(unused_imports)] pub use simulator_iter;

#[macro_export]
macro_rules! simulator_iter_real {
    ($simulator:ident, $position:ident, $node:ident, $body:expr) => {
        simulator_iter_with_filter!($simulator, $position, $node, $simulator.is_node_real(&$position), $body)
    };
    ($simulator:ident, $position:ident, $node:ident, t => $t:expr, $body:expr) => {
        simulator_iter_with_filter!($simulator, $position, $node, $simulator.is_node_real(&$position), t => $t, $body)
    };
}
#[allow(unused_imports)] pub use simulator_iter_real;

#[macro_export]
macro_rules! simulator_iter_virtual {
    ($simulator:ident, $position:ident, $node:ident, $body:expr) => {
        simulator_iter_with_filter!($simulator, $position, $node, $simulator.is_node_virtual(&$position), $body)
    };
    ($simulator:ident, $position:ident, $node:ident, t => $t:expr, $body:expr) => {
        simulator_iter_with_filter!($simulator, $position, $node, $simulator.is_node_virtual(&$position), t => $t, $body)
    };
}
#[allow(unused_imports)] pub use simulator_iter_virtual;

#[macro_export]
macro_rules! simulator_iter_mut_with_filter {
    ($simulator:ident, $position:ident, $node:ident, $filter:expr, $body:expr) => {
        simulator_iter_loop!($simulator, $position, $node, $filter, $body, 0, $simulator.height, 1, $simulator.get_node_mut_unwrap($position))
    };
    ($simulator:ident, $position:ident, $node:ident, $filter:expr, t => $t:expr, $body:expr) => {
        simulator_iter_loop!($simulator, $position, $node, $filter, $body, $t, $t+1, 1, $simulator.get_node_mut_unwrap($position))
    };
}
#[allow(unused_imports)] pub use simulator_iter_mut_with_filter;

#[macro_export]
macro_rules! simulator_iter_mut {
    ($simulator:ident, $position:ident, $node:ident, $body:expr) => {
        simulator_iter_mut_with_filter!($simulator, $position, $node, $simulator.is_node_exist(&$position), $body)
    };
    ($simulator:ident, $position:ident, $node:ident, t => $t:expr, $body:expr) => {
        simulator_iter_mut_with_filter!($simulator, $position, $node, $simulator.is_node_exist(&$position), t => $t, $body)
    };
}
#[allow(unused_imports)] pub use simulator_iter_mut;

#[macro_export]
macro_rules! simulator_iter_mut_real {
    ($simulator:ident, $position:ident, $node:ident, $body:expr) => {
        simulator_iter_mut_with_filter!($simulator, $position, $node, $simulator.is_node_real(&$position), $body)
    };
    ($simulator:ident, $position:ident, $node:ident, t => $t:expr, $body:expr) => {
        simulator_iter_mut_with_filter!($simulator, $position, $node, $simulator.is_node_real(&$position), t => $t, $body)
    };
}
#[allow(unused_imports)] pub use simulator_iter_mut_real;

#[macro_export]
macro_rules! simulator_iter_mut_virtual {
    ($simulator:ident, $position:ident, $node:ident, $body:expr) => {
        simulator_iter_mut_with_filter!($simulator, $position, $node, $simulator.is_node_virtual(&$position), $body)
    };
    ($simulator:ident, $position:ident, $node:ident, t => $t:expr, $body:expr) => {
        simulator_iter_mut_with_filter!($simulator, $position, $node, $simulator.is_node_virtual(&$position), t => $t, $body)
    };
}
#[allow(unused_imports)] pub use simulator_iter_mut_virtual;

/// faster way creating `Position`
#[macro_export]
macro_rules! pos {
    ($t:expr, $i:expr, $j:expr) => {
        Position::new($t, $i, $j)
    };
}
#[allow(unused_imports)] pub use pos;
