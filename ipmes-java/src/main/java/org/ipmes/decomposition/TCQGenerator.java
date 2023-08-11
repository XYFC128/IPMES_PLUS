package org.ipmes.decomposition;

import org.ipmes.pattern.TemporalRelation;
import org.ipmes.pattern.PatternEdge;
import org.ipmes.pattern.PatternGraph;

import java.util.*;

public class TCQGenerator {

    TemporalRelation temporalRelation;
    PatternGraph spatialRelation;
    // TCQueryId, relation of edges
    ArrayList<TCQueryRelation>[] TCQRelation;

    public TCQGenerator(TemporalRelation temporalRelation, PatternGraph spatialRelation) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
    }

    /**
     * Decompose the possibly non-TC pattern into TC-Queries
     *
     * @return TC-Queries
     */
    public ArrayList<TCQuery> decompose() {
        // DFS to generate TC sub-queries
        ArrayList<TCQuery> subQueries = new ArrayList<>();
        ArrayList<PatternEdge> parents = new ArrayList<>();
        for (PatternEdge edge : this.spatialRelation.getEdges()) {
            generateTCQueries(edge, parents, subQueries);
        }

        ArrayList<TCQuery> selected = selectTCSubQueries(subQueries);
        for (int i = 0; i < selected.size(); ++i) {
            selected.get(i).setId(i);
        }

        this.TCQRelation = genRelations(selected);
        return selected;
    }

    public ArrayList<TCQueryRelation>[] getTCQRelation() {
        return this.TCQRelation;
    }

    boolean hasSharedNode(PatternEdge edge, ArrayList<PatternEdge> parents) {
        if (parents.isEmpty())
            return true;
        for (PatternEdge parent : parents) {
            // checking 1.
            if (!this.spatialRelation.getSharedNodes(edge.getId(), parent.getId()).isEmpty()) {
                return true;
            }
        }
        return false;
    }

    /**
     * DFS on {@link TemporalRelation} to find out all possible TC sub-queries
     * starting at
     * the given node in the graph.
     * <p>
     * When encounter a new node, we check the following constraints:
     * <ol>
     * <li>new edge need to have share node with TC sub-query</li>
     * <li>follow temporal rules</li>
     * </ol>
     * If all checks passed, create a new TC sub-query and append to the results.
     * </p>
     * 
     * @param cur     current PatternEdge
     * @param parents the traversed path
     * @param results an array list to store the results
     */
    private void generateTCQueries(PatternEdge cur, ArrayList<PatternEdge> parents, ArrayList<TCQuery> results) {
        if (!hasSharedNode(cur, parents))
            return;
        parents.add(cur);
        results.add(new TCQuery(new ArrayList<>(parents)));
        for (Integer eid : this.temporalRelation.getChildren(cur.getId())) {
            generateTCQueries(this.spatialRelation.getEdge(eid), parents, results);
        }
        parents.remove(parents.size() - 1);
    }

    boolean containsSelectedEdge(TCQuery subQuery, boolean[] isEdgeSelected) {
        for (PatternEdge e : subQuery.getEdges()) {
            if (isEdgeSelected[e.getId()]) {
                return true;
            }
        }
        return false;
    }

    /**
     * Greedy select the longest possible TC-Query until all edges in the
     * pattern are selected.
     * <p>
     * Each edge will only be in one TC-Query
     * </p>
     * 
     * @param subQueries all the possible TC sub-queries
     * @return all the TC-Queries
     */
    ArrayList<TCQuery> selectTCSubQueries(ArrayList<TCQuery> subQueries) {
        // sort in decreasing size
        subQueries.sort((Q1, Q2) -> (Q2.numEdges() - Q1.numEdges()));

        ArrayList<TCQuery> selectedTCQ = new ArrayList<>();
        boolean[] isEdgeSelected = new boolean[this.spatialRelation.numEdges()];
        Arrays.fill(isEdgeSelected, false);
        for (TCQuery subQuery : subQueries) {
            if (containsSelectedEdge(subQuery, isEdgeSelected))
                continue;

            for (PatternEdge e : subQuery.getEdges())
                isEdgeSelected[e.getId()] = true;
            selectedTCQ.add(subQuery);
        }

        return selectedTCQ;
    }

    boolean hasRelations(PatternEdge edge1, PatternEdge edge2) {
        return this.temporalRelation.getParents(edge1.getId()).contains(edge2.getId())
                || this.temporalRelation.getChildren(edge1.getId()).contains(edge2.getId())
                || (!this.spatialRelation.getSharedNodes(edge1.getId(), edge2.getId()).isEmpty());
    }

    /**
     * Generate relations for all selected TC Query. This method will collect all
     * relations
     * related with all edges in a tc Query, including temporal relations and
     * spatial relations.
     * 
     * @param selected selected TC-Query
     * @return an array of relations, i-th element is the relations for i-th
     *         TC-Query
     */
    ArrayList<TCQueryRelation>[] genRelations(ArrayList<TCQuery> selected) {
        ArrayList<TCQueryRelation>[] relations = (ArrayList<TCQueryRelation>[]) new ArrayList[selected.size()];
        for (int i = 0; i < relations.length; i++) {
            relations[i] = new ArrayList<>();
            for (int j = 0; j < relations.length; j++) {
                if (i == j)
                    continue;
                for (PatternEdge edge1 : selected.get(i).edges) {
                    for (PatternEdge edge2 : selected.get(j).edges) {
                        if (hasRelations(edge1, edge2)) {
                            TCQueryRelation tempRelation = new TCQueryRelation();
                            tempRelation.idOfResult = edge1.getId();
                            tempRelation.idOfEntry = edge2.getId();
                            relations[i].add(tempRelation);
                        }
                    }
                }
            }
        }
        return relations;
    }
}
