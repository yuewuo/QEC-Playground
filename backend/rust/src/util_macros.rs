#[macro_export]
macro_rules! simulator_iter_with_filter {
    ($simulator:ident, $position:ident, $node:ident, $filter:expr, $body:expr) => {
        for $position in $simulator.position_iter() {
            if $filter {
                let $node = $simulator.get_node_unwrap(&$position);
                $body
            }
        }
    };
    ($simulator:ident, $position:ident, $node:ident, $filter:expr, t => $t:expr, $body:expr) => {
        for $position in $simulator.position_iter_t($t) {
            if $filter {
                let $node = $simulator.get_node_unwrap(&$position);
                $body
            }
        }
    };
}
#[allow(unused_imports)] pub use simulator_iter_with_filter;

#[macro_export]
macro_rules! simulator_iter {
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
        for $position in $simulator.position_iter() {
            if $filter {
                let $node = $simulator.get_node_mut_unwrap(&$position);
                $body
            }
        }
    };
    ($simulator:ident, $position:ident, $node:ident, $filter:expr, t => $t:expr, $body:expr) => {
        for $position in $simulator.position_iter_t($t) {
            if $filter {
                let $node = $simulator.get_node_mut_unwrap(&$position);
                $body
            }
        }
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
