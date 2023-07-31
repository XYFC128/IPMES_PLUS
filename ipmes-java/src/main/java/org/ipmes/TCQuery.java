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
    ArrayList<Integer> edges;
    ArrayList<Integer> nodes;
    PatternGraph SpatialRelation;

    public TCQuery(ArrayList<Integer> edges, PatternGraph SpatialRelation) {
        this.edges = edges;
        this.SpatialRelation = SpatialRelation;

        HashSet<Integer> nodes = new HashSet<>();
        for (Integer eid : edges) {
            PatternEdge e = SpatialRelation.getEdge(eid);
            nodes.add(e.getStartId());
            nodes.add(e.getEndId());
        }
        this.nodes = new ArrayList<>(nodes);
    }

    /**
     * Get the list of edges in the TC-Query. Ordered by temporal order.
     * @return list of pattern edge id in temporal order
     */
    public ArrayList<Integer> getQueryEdges() {
        return this.edges;
    }

    /**
     * Get the pattern nodes in the TC-Query. Ordered by their id.
     * @return list of sorted pattern node id
     */
    public ArrayList<Integer> getQueryNodes() {
        return this.nodes;
    }

    // return an arraylist contains every share node of the edge in this query
    public ArrayList<Integer> getInQueryDependencies(Integer eid) {
        ArrayList<Integer> ret = new ArrayList<Integer>();
        for (int i : this.edges) {
            if (i != eid && (!this.SpatialRelation.getSharedNodes(i, eid).isEmpty()))
                ret.add(i);
        }
        return ret;
    }

    // getter of TCQueryID
    Integer getTCQueryID() {
        return id;
    }

    // setter of TCQueryID
    void setTCQueryID(Integer id) {
        this.id = id;
    }
}
