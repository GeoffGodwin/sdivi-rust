#!/usr/bin/env python3
"""Generate leiden-graph fixture files for the sdi-rust verify-leiden test suite.

Produces three fixtures (small/medium/large) as ring-of-cliques graphs:
- small:  50 nodes, 5 cliques of 10
- medium: 500 nodes, 10 cliques of 50
- large:  5000 nodes, 50 cliques of 100

Each fixture directory contains:
  adjacency.json   — list of [src, dst] undirected edge pairs
  metadata.json    — node_count, edge_count, reference_modularity,
                     reference_community_count

When leidenalg is available (pip install leidenalg igraph), the reference
values are computed by running leidenalg with seed=42.  Otherwise, the
theoretical optimal modularity for the planted partition is used as the
reference.

Usage:
  python tools/generate-leiden-fixtures.py           # uses theoretical values
  python tools/generate-leiden-fixtures.py --leiden  # runs leidenalg
"""

import json
import math
import os
import sys
import argparse


def ring_of_cliques_edges(k: int, s: int) -> list[tuple[int, int]]:
    """Generate edge list for a ring-of-k-cliques with s nodes each."""
    edges = []
    # Internal clique edges (all pairs)
    for clique in range(k):
        base = clique * s
        for i in range(s):
            for j in range(i + 1, s):
                edges.append((base + i, base + j))
    # Bridge edges forming a ring
    for clique in range(k):
        next_clique = (clique + 1) % k
        # Last node of clique -> first node of next clique
        edges.append((clique * s + s - 1, next_clique * s))
    return edges


def theoretical_modularity(k: int, s: int, edges: list) -> float:
    """Compute the theoretical optimal modularity for a ring-of-cliques."""
    m = len(edges)
    if m == 0:
        return 0.0

    # Build degree array
    n = k * s
    degree = [0] * n
    for u, v in edges:
        degree[u] += 1
        degree[v] += 1

    # Compute modularity assuming each clique is one community
    q = 0.0
    for clique in range(k):
        base = clique * s
        nodes = list(range(base, base + s))
        # Internal edges (L_C)
        l_c = s * (s - 1) / 2
        # Sum of degrees (Sigma_C)
        sigma_c = sum(degree[v] for v in nodes)
        q += l_c / m - (sigma_c / (2 * m)) ** 2

    return q


def leidenalg_modularity(n: int, edges: list) -> tuple[float, int]:
    """Run leidenalg and return (modularity, community_count)."""
    try:
        import igraph as ig
        import leidenalg
    except ImportError:
        print("leidenalg/igraph not available; using theoretical values", file=sys.stderr)
        return None, None

    g = ig.Graph(n=n, edges=edges)
    partition = leidenalg.find_partition(
        g,
        leidenalg.ModularityVertexPartition,
        seed=42,
    )
    return partition.modularity, len(partition)


def write_fixture(out_dir: str, k: int, s: int, use_leidenalg: bool) -> None:
    """Generate and write a single fixture."""
    os.makedirs(out_dir, exist_ok=True)
    n = k * s
    edges = ring_of_cliques_edges(k, s)
    edge_count = len(edges)

    if use_leidenalg:
        ref_mod, ref_count = leidenalg_modularity(n, edges)
        if ref_mod is None:
            ref_mod = theoretical_modularity(k, s, edges)
            ref_count = k
    else:
        ref_mod = theoretical_modularity(k, s, edges)
        ref_count = k

    # Write adjacency.json
    with open(os.path.join(out_dir, "adjacency.json"), "w") as f:
        json.dump([[u, v] for u, v in edges], f, separators=(",", ":"))

    # Write metadata.json
    meta = {
        "node_count": n,
        "edge_count": edge_count,
        "reference_modularity": round(ref_mod, 6),
        "reference_community_count": ref_count,
    }
    with open(os.path.join(out_dir, "metadata.json"), "w") as f:
        json.dump(meta, f, indent=2)

    print(
        f"  {out_dir}: n={n}, e={edge_count}, "
        f"Q={ref_mod:.4f}, k={ref_count}"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--leiden",
        action="store_true",
        help="Use leidenalg to compute reference values (requires pip install leidenalg igraph)",
    )
    args = parser.parse_args()

    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    fixtures_root = os.path.join(repo_root, "tests", "fixtures", "leiden-graphs")

    print("Generating leiden-graph fixtures …")

    specs = [
        ("small",  5,  10),   # 50 nodes, 5 communities of 10
        ("medium", 10, 50),   # 500 nodes, 10 communities of 50
        ("large",  50, 100),  # 5000 nodes, 50 communities of 100
    ]

    for name, k, s in specs:
        out_dir = os.path.join(fixtures_root, name)
        write_fixture(out_dir, k, s, args.leiden)

    print("Done.")


if __name__ == "__main__":
    main()
