use crate::analysis::{
    analyze_network_stats, analyze_stake_distribution, check_triangle_inequality, delta_q_analysis,
};
use crate::models::{Severity, Topology};

pub fn generate_report(
    topology: &Topology,
    filename: &str,
    delta_q: bool,
    verbose: bool,
) -> String {
    let mut report = String::new();
    let mut issues = Vec::new();

    // Run analyses
    let network_stats = analyze_network_stats(topology);
    let stake_stats = analyze_stake_distribution(topology);
    issues.extend(check_triangle_inequality(topology));

    // Sort issues by severity and node id
    issues.sort_by_key(|issue| (-(issue.severity as i32), issue.node.clone()));

    // Generate report sections
    report.push_str("# Topology Analysis Report\n\n");
    report.push_str(&format!("Analysis of: {}\n\n", filename));

    // Add network statistics section
    report.push_str("## Network Statistics\n\n");
    report.push_str("| Metric | Value |\n");
    report.push_str("|--------|-------|\n");
    report.push_str(&format!(
        "| Total nodes | {} |\n",
        network_stats.total_nodes
    ));
    report.push_str(&format!(
        "| Block producers | {} |\n",
        network_stats.block_producers
    ));
    report.push_str(&format!(
        "| Relay nodes | {} |\n",
        network_stats.relay_nodes
    ));
    report.push_str(&format!(
        "| Total connections | {} |\n",
        network_stats.total_connections
    ));
    report.push_str(&format!(
        "| Network diameter | {} hops |\n",
        network_stats.network_diameter
    ));
    report.push_str(&format!(
        "| Average connections per node | {:.2} |\n",
        network_stats.avg_connections
    ));
    report.push_str(&format!(
        "| Clustering coefficient | {:.3} |\n",
        network_stats.clustering_coefficient
    ));
    report.push_str(&format!(
        "| Average latency | {:.2} ms |\n",
        network_stats.avg_latency_ms
    ));
    report.push_str(&format!(
        "| Maximum latency | {:.2} ms |\n",
        network_stats.max_latency_ms
    ));
    report.push_str(&format!(
        "| Stake-weighted latency | {:.2} ms |\n",
        network_stats.stake_weighted_latency_ms
    ));
    report.push_str(&format!(
        "| Bidirectional connections | {} |\n",
        network_stats.bidirectional_connections
    ));
    report.push_str(&format!(
        "| Asymmetry ratio | {:.2}% |\n\n",
        network_stats.asymmetry_ratio * 100.0
    ));

    // Add stake distribution section
    report.push_str("## Stake Distribution\n\n");
    report.push_str("| Metric | Value |\n");
    report.push_str("|--------|-------|\n");
    report.push_str(&format!("| Total stake | {} |\n", stake_stats.total_stake));
    report.push_str(&format!(
        "| Gini coefficient | {:.3} |\n\n",
        stake_stats.gini_coefficient
    ));

    report.push_str("### Top 5 Stake Holders\n\n");
    report.push_str("| Node | Stake | % of Total |\n");
    report.push_str("|------|--------|------------|\n");
    for (node, stake) in stake_stats.top_stake_holders {
        let percentage = (stake as f64 / stake_stats.total_stake as f64) * 100.0;
        report.push_str(&format!("| {} | {} | {:.2}% |\n", node, stake, percentage));
    }
    report.push_str("\n");

    report.push_str("### Geographic Stake Distribution\n\n");
    report.push_str("| Region | Nodes | Total Stake | % of Network |\n");
    report.push_str("|---------|--------|-------------|-------------|\n");
    for (region, stats) in stake_stats.geographic_distribution {
        report.push_str(&format!(
            "| {} | {} | {} | {:.2}% |\n",
            region, stats.node_count, stats.total_stake, stats.percentage
        ));
    }
    report.push_str("\n");

    // Add network reliability section
    if !network_stats.critical_nodes.is_empty() {
        report.push_str("## Network Reliability\n\n");
        report.push_str("The following nodes, if removed, would isolate significant stake:\n\n");
        report.push_str("| Node | Isolated Stake | % of Total Stake |\n");
        report.push_str("|------|----------------|------------------|\n");
        for node in &network_stats.critical_nodes {
            report.push_str(&format!(
                "| {} | {} | {:.2}% |\n",
                node.node, node.isolated_stake, node.percentage_of_total
            ));
        }
        report.push_str("\n");
    }

    // Add geographic validation section
    report.push_str("## Geographic Validation\n\n");
    let geo_issues: Vec<_> = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .collect();

    if geo_issues.is_empty() {
        report.push_str("✅ No geographic violations found\n\n");
    } else {
        report.push_str("### Path Length Violations\n\n");
        for issue in geo_issues {
            report.push_str(&format!("❌ {}\n", issue.message));
            report.push_str(&format!("   Suggestion: {}\n\n", issue.suggestion));
        }
    }

    // Add hop analysis section if a start node is provided
    if delta_q {
        delta_q_analysis(topology, &mut report, verbose);
    }

    report
}
