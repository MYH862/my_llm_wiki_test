use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub link_count: usize,
    pub community: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityInfo {
    pub id: usize,
    pub node_count: usize,
    pub cohesion: f32,
    pub top_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub communities: Vec<CommunityInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub id: String,
    pub title: String,
    pub content: String,
    pub node_type: String,
    pub wikilinks: Vec<String>,
}

pub struct GraphService;

impl GraphService {
    pub fn build_graph(pages: Vec<WikiPage>) -> WikiGraph {
        if pages.is_empty() {
            return WikiGraph {
                nodes: vec![],
                edges: vec![],
                communities: vec![],
            };
        }

        let hidden_types: HashSet<&str> = ["query"].iter().cloned().collect();
        let page_map: HashMap<String, &WikiPage> = pages
            .iter()
            .filter(|p| !hidden_types.contains(p.node_type.as_str()))
            .map(|p| (p.id.clone(), p))
            .collect();

        let mut link_counts: HashMap<String, usize> = page_map.keys().map(|k| (k.clone(), 0)).collect();
        let mut raw_edges: Vec<(String, String)> = Vec::new();

        for page in page_map.values() {
            for target_raw in &page.wikilinks {
                if let Some(target_id) = resolve_target(target_raw, &page_map) {
                    if target_id != page.id {
                        raw_edges.push((page.id.clone(), target_id.clone()));
                        *link_counts.entry(page.id.clone()).or_insert(0) += 1;
                        *link_counts.entry(target_id).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut seen_edges: HashSet<String> = HashSet::new();
        let mut deduped_edges: Vec<(String, String)> = Vec::new();
        for (source, target) in &raw_edges {
            let key = format!("{}:::{}", source, target);
            let reverse_key = format!("{}:::{}", target, source);
            if !seen_edges.contains(&key) && !seen_edges.contains(&reverse_key) {
                seen_edges.insert(key);
                deduped_edges.push((source.clone(), target.clone()));
            }
        }

        let edges_with_weights: Vec<GraphEdge> = deduped_edges
            .iter()
            .map(|(source, target)| GraphEdge {
                source: source.clone(),
                target: target.clone(),
                weight: 1.0,
            })
            .collect();

        let prelim_nodes: Vec<(String, String, String, usize)> = page_map
            .values()
            .map(|p| (
                p.id.clone(),
                p.title.clone(),
                p.node_type.clone(),
                link_counts.get(&p.id).copied().unwrap_or(0),
            ))
            .collect();

        let (assignments, communities) = detect_communities(&prelim_nodes, &edges_with_weights);

        let nodes: Vec<GraphNode> = page_map
            .values()
            .map(|p| GraphNode {
                id: p.id.clone(),
                label: p.title.clone(),
                node_type: p.node_type.clone(),
                link_count: link_counts.get(&p.id).copied().unwrap_or(0),
                community: assignments.get(&p.id).copied().unwrap_or(0),
            })
            .collect();

        WikiGraph {
            nodes,
            edges: edges_with_weights,
            communities,
        }
    }
}

fn resolve_target<'a>(raw: &str, page_map: &'a HashMap<String, &'a WikiPage>) -> Option<String> {
    if page_map.contains_key(raw) {
        return Some(raw.to_string());
    }

    let normalized = raw.to_lowercase().replace(" ", "-");
    for id in page_map.keys() {
        if id.to_lowercase() == normalized {
            return Some(id.clone());
        }
        if id.to_lowercase() == raw.to_lowercase() {
            return Some(id.clone());
        }
        if id.to_lowercase().replace(" ", "-") == normalized {
            return Some(id.clone());
        }
    }

    None
}

fn detect_communities(
    nodes: &[(String, String, String, usize)],
    edges: &[GraphEdge],
) -> (HashMap<String, usize>, Vec<CommunityInfo>) {
    if nodes.is_empty() {
        return (HashMap::new(), vec![]);
    }

    let mut graph: UnGraph<(), f32> = UnGraph::new_undirected();
    let node_indices: HashMap<String, NodeIndex> = nodes
        .iter()
        .map(|(id, _, _, _)| (id.clone(), graph.add_node(())))
        .collect();

    for edge in edges {
        if let (Some(&source_idx), Some(&target_idx)) = (
            node_indices.get(&edge.source),
            node_indices.get(&edge.target),
        ) {
            if !graph.contains_edge(source_idx, target_idx) {
                graph.add_edge(source_idx, target_idx, edge.weight);
            }
        }
    }

    let assignments = simple_louvain(&graph, &node_indices, nodes);

    let mut groups: HashMap<usize, Vec<String>> = HashMap::new();
    for (node_id, comm_id) in &assignments {
        groups.entry(*comm_id).or_insert_with(Vec::new).push(node_id.clone());
    }

    let edge_set: HashSet<String> = edges
        .iter()
        .flat_map(|e| {
            vec![
                format!("{}:::{}", e.source, e.target),
                format!("{}:::{}", e.target, e.source),
            ]
        })
        .collect();

    let node_info: HashMap<String, (String, usize)> = nodes
        .iter()
        .map(|(id, label, _, link_count)| (id.clone(), (label.clone(), *link_count)))
        .collect();

    let mut communities: Vec<CommunityInfo> = Vec::new();
    for (comm_id, member_ids) in &groups {
        let n = member_ids.len();
        let mut intra_edges = 0;
        for i in 0..member_ids.len() {
            for j in (i + 1)..member_ids.len() {
                if edge_set.contains(&format!("{}:::{}", member_ids[i], member_ids[j])) {
                    intra_edges += 1;
                }
            }
        }
        let possible_edges = if n > 1 { n * (n - 1) / 2 } else { 1 };
        let cohesion = intra_edges as f32 / possible_edges as f32;

        let mut sorted_members = member_ids.clone();
        sorted_members.sort_by(|a, b| {
            let a_count = node_info.get(b).map(|(_, c)| *c).unwrap_or(0);
            let b_count = node_info.get(a).map(|(_, c)| *c).unwrap_or(0);
            a_count.cmp(&b_count)
        });

        let top_nodes: Vec<String> = sorted_members
            .iter()
            .take(5)
            .filter_map(|id| node_info.get(id).map(|(label, _)| label.clone()))
            .collect();

        communities.push(CommunityInfo {
            id: *comm_id,
            node_count: n,
            cohesion,
            top_nodes,
        });
    }

    communities.sort_by(|a, b| b.node_count.cmp(&a.node_count));

    let mut id_remap: HashMap<usize, usize> = HashMap::new();
    for (idx, comm) in communities.iter_mut().enumerate() {
        id_remap.insert(comm.id, idx);
        comm.id = idx;
    }

    let final_assignments: HashMap<String, usize> = assignments
        .into_iter()
        .map(|(node_id, old_id)| (node_id, id_remap.get(&old_id).copied().unwrap_or(0)))
        .collect();

    (final_assignments, communities)
}

fn simple_louvain(
    graph: &UnGraph<(), f32>,
    node_indices: &HashMap<String, NodeIndex>,
    nodes: &[(String, String, String, usize)],
) -> HashMap<String, usize> {
    let mut community: HashMap<NodeIndex, usize> = node_indices
        .values()
        .enumerate()
        .map(|(i, &idx)| (idx, i))
        .collect();

    let mut improved = true;
    let mut iterations = 0;
    let max_iterations = 10;

    while improved && iterations < max_iterations {
        improved = false;
        iterations += 1;

        for (node_id, &node_idx) in node_indices {
            let current_comm = *community.get(&node_idx).unwrap_or(&0);
            let mut best_comm = current_comm;
            let mut best_modularity_gain = 0.0;

            let neighbor_communities: HashSet<usize> = graph
                .edges(node_idx)
                .filter_map(|edge| {
                    let other = if edge.source() == node_idx {
                        edge.target()
                    } else {
                        edge.source()
                    };
                    community.get(&other).copied()
                })
                .collect();

            for &comm in &neighbor_communities {
                if comm == current_comm {
                    continue;
                }

                let gain = calculate_modularity_gain(graph, node_idx, comm, &community);
                if gain > best_modularity_gain {
                    best_modularity_gain = gain;
                    best_comm = comm;
                }
            }

            if best_comm != current_comm {
                community.insert(node_idx, best_comm);
                improved = true;
            }
        }
    }

    let mut result: HashMap<String, usize> = HashMap::new();
    for (node_id, &idx) in node_indices {
        if let Some(&comm) = community.get(&idx) {
            result.insert(node_id.clone(), comm);
        }
    }

    result
}

fn calculate_modularity_gain(
    graph: &UnGraph<(), f32>,
    node: NodeIndex,
    target_community: usize,
    community: &HashMap<NodeIndex, usize>,
) -> f32 {
    let mut internal_edges = 0.0;
    let mut total_edges = 0.0;

    for edge in graph.edges(node) {
        let other = if edge.source() == node {
            edge.target()
        } else {
            edge.source()
        };

        let weight = edge.weight();
        total_edges += weight;

        if community.get(&other) == Some(&target_community) {
            internal_edges += weight;
        }
    }

    if total_edges == 0.0 {
        return 0.0;
    }

    let community_size = community.values().filter(|&&c| c == target_community).count() as f32;
    let total_nodes = community.len() as f32;

    if total_nodes <= 1.0 || community_size <= 1.0 {
        return 0.0;
    }

    let expected = community_size / total_nodes;
    let actual = internal_edges / total_edges;

    actual - expected
}

pub fn find_surprising_connections(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    _communities: &[CommunityInfo],
    limit: usize,
) -> Vec<SurprisingConnection> {
    let node_map: HashMap<String, &GraphNode> = nodes.iter().map(|n| (n.id.clone(), n)).collect();
    let max_degree = nodes.iter().map(|n| n.link_count).max().unwrap_or(1) as f32;

    let structural_ids: HashSet<&str> = ["index", "log", "overview"].iter().cloned().collect();

    let mut scored: Vec<SurprisingConnection> = Vec::new();

    for edge in edges {
        if let (Some(source), Some(target)) = (node_map.get(&edge.source), node_map.get(&edge.target)) {
            if structural_ids.contains(source.id.as_str()) || structural_ids.contains(target.id.as_str()) {
                continue;
            }

            let mut score = 0;
            let mut reasons: Vec<String> = Vec::new();

            if source.community != target.community {
                score += 3;
                reasons.push("crosses community boundary".to_string());
            }

            if source.node_type != target.node_type {
                let distant_pairs: HashSet<(&str, &str)> = [
                    ("source", "concept"), ("concept", "source"),
                    ("source", "synthesis"), ("synthesis", "source"),
                ].iter().cloned().collect();

                let pair = (source.node_type.as_str(), target.node_type.as_str());
                if distant_pairs.contains(&pair) {
                    score += 2;
                    reasons.push(format!("connects {} to {}", source.node_type, target.node_type));
                } else {
                    score += 1;
                    reasons.push("different types".to_string());
                }
            }

            let min_deg = (source.link_count.min(target.link_count)) as f32;
            let max_deg = (source.link_count.max(target.link_count)) as f32;
            if min_deg <= 2.0 && max_deg >= max_degree * 0.5 {
                score += 2;
                reasons.push("peripheral node links to hub".to_string());
            }

            if edge.weight < 2.0 && edge.weight > 0.0 {
                score += 1;
                reasons.push("weak but present connection".to_string());
            }

            if score >= 3 && !reasons.is_empty() {
                let mut key_parts = vec![source.id.clone(), target.id.clone()];
                key_parts.sort();
                let key = key_parts.join(":::");

                scored.push(SurprisingConnection {
                    source: (*source).clone(),
                    target: (*target).clone(),
                    score,
                    reasons,
                    key,
                });
            }
        }
    }

    scored.sort_by(|a, b| b.score.cmp(&a.score));
    scored.into_iter().take(limit).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurprisingConnection {
    pub source: GraphNode,
    pub target: GraphNode,
    pub score: usize,
    pub reasons: Vec<String>,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    #[serde(rename = "type")]
    pub gap_type: String,
    pub title: String,
    pub description: String,
    pub node_ids: Vec<String>,
    pub suggestion: String,
}

pub fn detect_knowledge_gaps(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    communities: &[CommunityInfo],
    limit: usize,
) -> Vec<KnowledgeGap> {
    let mut gaps: Vec<KnowledgeGap> = Vec::new();
    let node_map: HashMap<String, &GraphNode> = nodes.iter().map(|n| (n.id.clone(), n)).collect();

    let isolated_nodes: Vec<&GraphNode> = nodes
        .iter()
        .filter(|n| n.link_count <= 1 && n.node_type != "overview" && n.id != "index" && n.id != "log")
        .collect();

    if !isolated_nodes.is_empty() {
        let top_isolated: Vec<&GraphNode> = isolated_nodes.iter().take(5).cloned().collect();
        let description_base = top_isolated
            .iter()
            .map(|n| n.label.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let description = if isolated_nodes.len() > 5 {
            format!("{} and {} more", description_base, isolated_nodes.len() - 5)
        } else {
            description_base
        };

        gaps.push(KnowledgeGap {
            gap_type: "isolated-node".to_string(),
            title: format!(
                "{} isolated page{}",
                isolated_nodes.len(),
                if isolated_nodes.len() > 1 { "s" } else { "" }
            ),
            description,
            node_ids: isolated_nodes.iter().map(|n| n.id.clone()).collect(),
            suggestion: "These pages have few or no connections. Consider adding [[wikilinks]] to related pages, or research to expand their content.".to_string(),
        });
    }

    for comm in communities {
        if comm.cohesion < 0.15 && comm.node_count >= 3 {
            gaps.push(KnowledgeGap {
                gap_type: "sparse-community".to_string(),
                title: format!(
                    "Sparse cluster: {}",
                    comm.top_nodes.first().unwrap_or(&format!("Community {}", comm.id))
                ),
                description: format!(
                    "{} pages with cohesion {:.2} — internal connections are weak.",
                    comm.node_count, comm.cohesion
                ),
                node_ids: nodes
                    .iter()
                    .filter(|n| n.community == comm.id)
                    .map(|n| n.id.clone())
                    .collect(),
                suggestion: "This knowledge area lacks internal cross-references. Consider adding links between these pages or researching to fill gaps.".to_string(),
            });
        }
    }

    let mut community_neighbors: HashMap<String, HashSet<usize>> = nodes
        .iter()
        .map(|n| (n.id.clone(), HashSet::new()))
        .collect();

    for edge in edges {
        if let (Some(source_node), Some(target_node)) = (node_map.get(&edge.source), node_map.get(&edge.target)) {
            community_neighbors
                .entry(edge.source.clone())
                .or_insert_with(HashSet::new)
                .insert(target_node.community);
            community_neighbors
                .entry(edge.target.clone())
                .or_insert_with(HashSet::new)
                .insert(source_node.community);
        }
    }

    let structural_ids: HashSet<&str> = ["index", "log", "overview"].iter().cloned().collect();

    let mut bridge_nodes: Vec<&GraphNode> = nodes
        .iter()
        .filter(|n| {
            if structural_ids.contains(n.id.as_str()) {
                return false;
            }
            community_neighbors
                .get(&n.id)
                .map(|comms| comms.len() >= 3)
                .unwrap_or(false)
        })
        .collect();

    bridge_nodes.sort_by(|a, b| {
        let a_comms = community_neighbors.get(&a.id).map(|c| c.len()).unwrap_or(0);
        let b_comms = community_neighbors.get(&b.id).map(|c| c.len()).unwrap_or(0);
        b_comms.cmp(&a_comms)
    });

    for bridge in bridge_nodes.iter().take(3) {
        let comm_count = community_neighbors.get(&bridge.id).map(|c| c.len()).unwrap_or(0);
        gaps.push(KnowledgeGap {
            gap_type: "bridge-node".to_string(),
            title: format!("Key bridge: {}", bridge.label),
            description: format!(
                "Connects {} different knowledge clusters. This is a critical junction in your wiki.",
                comm_count
            ),
            node_ids: vec![bridge.id.clone()],
            suggestion: "This page bridges multiple knowledge areas. Ensure it's well-maintained — if it's thin, expanding it will strengthen your entire wiki.".to_string(),
        });
    }

    gaps.into_iter().take(limit).collect()
}
