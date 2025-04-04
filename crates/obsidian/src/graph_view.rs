use gpui::{div, AnyElement, Bounds, Element, ElementId, IntoElement, Render, View, ViewContext, WindowContext};
use petgraph::graph::{Graph, NodeIndex};
use std::{collections::HashMap, path::PathBuf};

pub struct GraphView {
    graph: Graph<String, ()>,
    node_positions: HashMap<NodeIndex, (f64, f64)>,
    bounds: Bounds<f64>,
    zoom_level: f64,
}

impl GraphView {
    pub fn new(graph: Graph<String, ()>) -> Self {
        Self {
            graph,
            node_positions: HashMap::new(),
            bounds: Bounds {
                origin: gpui::Point::new(0.0, 0.0),
                size: gpui::Size::new(800.0, 600.0),
            },
            zoom_level: 1.0,
        }
    }

    fn calculate_layout(&mut self) {
        // Implement force-directed graph layout
        // This is a simplified version - we'll need to implement proper force-directed
        // layout algorithms like Fruchterman-Reingold for better visualization
        let node_count = self.graph.node_count();
        if node_count == 0 {
            return;
        }

        // Place nodes in a circle initially
        let radius = self.bounds.size.width.min(self.bounds.size.height) * 0.4;
        let center_x = self.bounds.size.width / 2.0;
        let center_y = self.bounds.size.height / 2.0;

        for (i, node_idx) in self.graph.node_indices().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (node_count as f64);
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            self.node_positions.insert(node_idx, (x, y));
        }
    }
}

impl Render for GraphView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        if self.node_positions.is_empty() {
            self.calculate_layout();
        }

        let mut elements = Vec::new();

        // Render edges
        for edge in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge) {
                if let (Some(&(x1, y1)), Some(&(x2, y2))) = (
                    self.node_positions.get(&source),
                    self.node_positions.get(&target),
                ) {
                    elements.push(
                        div()
                            .absolute()
                            .left(x1)
                            .top(y1)
                            .width(x2 - x1)
                            .height(y2 - y1)
                            .border_color(gpui::rgb(0x666666))
                            .border_width(1.0)
                            .into_any_element(),
                    );
                }
            }
        }

        // Render nodes
        for (node_idx, (x, y)) in &self.node_positions {
            if let Some(label) = self.graph.node_weight(*node_idx) {
                elements.push(
                    div()
                        .absolute()
                        .left(*x - 25.0)
                        .top(*y - 25.0)
                        .width(50.0)
                        .height(50.0)
                        .background(gpui::rgb(0x2a2a2a))
                        .border_radius(25.0)
                        .child(label)
                        .into_any_element(),
                );
            }
        }

        div()
            .size_full()
            .background(gpui::rgb(0x1a1a1a))
            .children(elements)
    }
}