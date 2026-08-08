use std::collections::HashMap;

use crate::core::connection::Connection;
use topo_sort::{SortResults, TopoSort};

pub struct ConnectionOrder {
    pub instruments: Vec<String>,
    pub is_partial: bool,
}

impl ConnectionOrder {
    pub fn new(connections: &Vec<Connection>) -> Self {
        let dependencies = Self::get_dependencies(connections);

        let topo_sort = Self::get_topo_sort(dependencies);

        match topo_sort.into_vec_nodes() {
            SortResults::Full(nodes) => Self {
                instruments: nodes,
                is_partial: false,
            },
            SortResults::Partial(nodes) => Self {
                instruments: nodes,
                is_partial: true,
            },
        }
    }

    fn get_dependencies(connections: &Vec<Connection>) -> HashMap<String, Vec<String>> {
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();

        for connection in connections {
            // the dictionary is "key" depends on "values"
            // so here target depends on source(s) so target is the key
            dependencies
                .entry(connection.target.instrument_name.to_string())
                .or_default()
                .push(connection.source.instrument_name.to_string());
        }

        // find top-level instruments
        for connection in connections {
            if !dependencies.contains_key(&connection.source.instrument_name) {
                dependencies
                    .entry(connection.source.instrument_name.to_string())
                    .or_default();
            }
        }

        dependencies
    }

    fn get_topo_sort(dependencies: HashMap<String, Vec<String>>) -> TopoSort<String> {
        let mut topo_sort = TopoSort::with_capacity(dependencies.len());

        for (key, value) in dependencies {
            topo_sort.insert(key, value);
        }

        topo_sort
    }
}

#[cfg(test)]
mod connection_order_tests {
    use super::*;
    use crate::core::connection::EndPoint;

    #[test]
    fn connection_order_orders_two_dependent_instruments() {
        let connection_1 = Connection {
            source: EndPoint {
                instrument_name: "A".to_string(),
                port: 0,
            },
            target: EndPoint {
                instrument_name: "B".to_string(),
                port: 0,
            },
        };
        let connection_2 = Connection {
            source: EndPoint {
                instrument_name: "B".to_string(),
                port: 0,
            },
            target: EndPoint {
                instrument_name: "C".to_string(),
                port: 0,
            },
        };
        let connection_3 = Connection {
            source: EndPoint {
                instrument_name: "A".to_string(),
                port: 1,
            },
            target: EndPoint {
                instrument_name: "C".to_string(),
                port: 1,
            },
        };

        let sut = ConnectionOrder::new(&vec![connection_1, connection_2, connection_3]);

        assert_eq!(sut.is_partial, false);
        assert_eq!(
            sut.instruments,
            vec!["A".to_string(), "B".to_string(), "C".to_string()]
        );
    }
}
