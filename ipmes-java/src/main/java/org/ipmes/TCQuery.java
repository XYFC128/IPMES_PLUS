package org.ipmes;

import java.util.*;

/**
 * Time-Connected Query.
 * <p>
 *     A TC-Query is a list of pattern edges where
 *     <ul>
 *         <li>All prefix of the list can form a weakly connected subgraph.</li>
 *         <li>
 *             An edge in the list depends on its previous edge. That means
 *             TC-Query is ordered by the temporal order of edges.
 *         </li>
 *     </ul>
 * </p>
 */
public class TCQuery {

    int id; // id for the TCQuery
    ArrayList<PatternEdge> edges;
    ArrayList<PatternNode> nodes;

    public TCQuery(ArrayList<PatternEdge> edges) {
        this.edges = edges;
        this.nodes = null;
    }

    public int numEdges() {
        return this.edges.size();
    }

    /**
     * Get the list of edges in the TC-Query. Ordered by temporal order.
     * @return list of pattern edge in temporal order
     */
    public ArrayList<PatternEdge> getEdges() {
        return this.edges;
    }

    public int numNodes() {
        return this.getNodes().size();
    }

    /**
     * Get the pattern nodes in the TC-Query. Ordered by their id.
     * <p>
     *     this.nodes is initially set to null, and will be created
     *     at the first call to this function. Because we generate
     *     a lot of TC-Queries during decomposition and the process
     *     doesn't need to use nodes, so this optimization can
     *     save us some time collecting nodes for unused TC-Queries.
     * </p>
     * <p>
     *     Also, store the nodes in TC-Query allows us to access it
     *     quickly without re-collecting it everytime we use it.
     * </p>
     * @return list of sorted pattern node
     */
    public ArrayList<PatternNode> getNodes() {
        if (this.nodes == null) {
            HashSet<PatternNode> nodes = new HashSet<>();
            for (PatternEdge edge : edges) {
                nodes.add(edge.getStartNode());
                nodes.add(edge.getEndNode());
            }
            this.nodes = new ArrayList<>(nodes);
            this.nodes.sort((n1, n2) -> (n1.getId() - n2.getId()));
        }
        return this.nodes;
    }

    // getter of TCQueryID
    public Integer getId() {
        return id;
    }

    // setter of TCQueryID
    public void setId(Integer id) {
        this.id = id;
    }
}
