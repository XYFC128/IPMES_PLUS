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
     * @return list of pattern edge id in temporal order
     */
    public ArrayList<PatternEdge> getEdges() {
        return this.edges;
    }

    public int numNodes() {
        return this.getNodes().size();
    }

    /**
     * Get the pattern nodes in the TC-Query. Ordered by their id.
     * @return list of sorted pattern node id
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
    Integer getId() {
        return id;
    }

    // setter of TCQueryID
    void setId(Integer id) {
        this.id = id;
    }
}
