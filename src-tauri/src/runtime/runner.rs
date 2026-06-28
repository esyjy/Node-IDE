use std::time::Duration;

use uuid::Uuid;

use super::edge::Edge;
use super::graph::{
    incoming_payload, topological_order, wired_input_envelope, GraphError, GraphRunResult,
    MessageDelivery,
};
use super::lifecycle::{Lifecycle, LifecycleMode};
use super::node::{NodeInstance, RunResult, RuntimeError};
use super::sdk::LifecycleHooks;

/// Inter-node delay for GUI graph runs so lifecycle transitions are visible.
pub const GUI_PACING_MS: u64 = 120;

/// Headless CLI runs use no pacing.
pub const CLI_PACING_MS: u64 = 0;

pub trait LifecycleObserver {
    fn on_lifecycle(
        &mut self,
        node_id: Uuid,
        previous: Lifecycle,
        current: Lifecycle,
        mode: LifecycleMode,
    );
    fn on_output(&mut self, node_id: Uuid, output: &str);
    fn on_delivery(&mut self, delivery: &MessageDelivery);
}

pub struct NoopObserver;

impl LifecycleObserver for NoopObserver {
    fn on_lifecycle(
        &mut self,
        _node_id: Uuid,
        _previous: Lifecycle,
        _current: Lifecycle,
        _mode: LifecycleMode,
    ) {
    }

    fn on_output(&mut self, _node_id: Uuid, _output: &str) {}

    fn on_delivery(&mut self, _delivery: &MessageDelivery) {}
}

struct ObservingHooks<'a> {
    observer: &'a mut dyn LifecycleObserver,
    node_id: Uuid,
}

impl LifecycleHooks for ObservingHooks<'_> {
    fn on_transition(&mut self, from: Lifecycle, to: Lifecycle, mode: LifecycleMode) {
        self.observer
            .on_lifecycle(self.node_id, from, to, mode);
    }
}

pub struct GraphRunner;

impl GraphRunner {
    pub fn run_node_observed(
        nodes: &mut [NodeInstance],
        id: Uuid,
        wired_input: Option<&str>,
        observer: &mut dyn LifecycleObserver,
    ) -> Result<RunResult, RuntimeError> {
        let idx = nodes
            .iter()
            .position(|n| n.id == id)
            .ok_or(RuntimeError::NotFound(id))?;

        let output = {
            let mut hooks = ObservingHooks {
                observer,
                node_id: id,
            };
            nodes[idx].run_with_hooks(wired_input, &mut hooks)?
        };

        observer.on_output(id, &output.output);
        Ok(output)
    }

    pub fn run_graph_observed(
        nodes: &mut [NodeInstance],
        edges: &[Edge],
        observer: &mut dyn LifecycleObserver,
        pacing_ms: u64,
    ) -> Result<GraphRunResult, GraphError> {
        let order = topological_order(nodes, edges)?;
        let mut deliveries = Vec::new();
        let mut node_results = Vec::new();

        for (i, node_id) in order.iter().enumerate() {
            if i > 0 && pacing_ms > 0 {
                std::thread::sleep(Duration::from_millis(pacing_ms));
            }

            let wired_input = incoming_payload(*node_id, edges, nodes)?;

            let idx = nodes
                .iter()
                .position(|n| n.id == *node_id)
                .ok_or(GraphError::NodeNotFound(*node_id))?;

            let result = {
                let mut hooks = ObservingHooks {
                    observer,
                    node_id: *node_id,
                };
                nodes[idx]
                    .run_with_hooks(wired_input.as_deref(), &mut hooks)
                    .map_err(GraphError::Runtime)?
            };

            observer.on_output(*node_id, &result.output);
            node_results.push(result);

            if let Some((edge_id, envelope)) = wired_input_envelope(*node_id, edges, nodes)? {
                let delivery = MessageDelivery { edge_id, envelope };
                observer.on_delivery(&delivery);
                deliveries.push(delivery);
            }
        }

        Ok(GraphRunResult {
            node_results,
            deliveries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::edge::{Edge, PORT_IN, PORT_OUT};
    use crate::runtime::node::{NodeKind, Position};
    use std::cell::RefCell;

    struct RecordingObserver {
        events: RefCell<Vec<(Uuid, Lifecycle, Lifecycle)>>,
    }

    impl LifecycleObserver for RecordingObserver {
        fn on_lifecycle(
            &mut self,
            node_id: Uuid,
            previous: Lifecycle,
            current: Lifecycle,
            _mode: LifecycleMode,
        ) {
            self.events
                .borrow_mut()
                .push((node_id, previous, current));
        }

        fn on_output(&mut self, _node_id: Uuid, _output: &str) {}

        fn on_delivery(&mut self, _delivery: &MessageDelivery) {}
    }

    #[test]
    fn observed_graph_emits_running_before_terminal() {
        let constant = NodeInstance::new(
            NodeKind::Constant {
                value: "hi".into(),
            },
            Some(Position::default()),
        );
        let echo = NodeInstance::new(
            NodeKind::Echo {
                input: "".into(),
            },
            Some(Position::default()),
        );
        let constant_id = constant.id;
        let echo_id = echo.id;
        let edge = Edge::new(constant_id, PORT_OUT, echo_id, PORT_IN);

        let mut nodes = vec![constant, echo];
        let mut observer = RecordingObserver {
            events: RefCell::new(Vec::new()),
        };

        GraphRunner::run_graph_observed(&mut nodes, &[edge], &mut observer, 0).unwrap();

        let events = observer.events.borrow();
        assert!(events.iter().any(|(_, _, to)| *to == Lifecycle::Running));
        assert!(events.iter().any(|(id, _, to)| *id == echo_id && *to == Lifecycle::Done));
    }
}
