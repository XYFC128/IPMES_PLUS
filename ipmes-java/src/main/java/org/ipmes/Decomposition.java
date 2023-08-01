package org.ipmes;

import java.util.*;

public class Decomposition {

    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;

    public Decomposition(DependencyGraph temporalRelation, PatternGraph spatialRelation) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
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
     * DFS on {@link DependencyGraph} to find out all possible TC sub-queries
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
     * @param cur current PatternEdge
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

    /**
     * Returns true if all edges in the given TC-Query are not selected.
     * @param subQuery the query
     * @param isEdgeSelected if an edge is selected, isEdgeSelected[edge id] will be true
     * @return true if all edges in the given TC-Query are not selected.
     */
    boolean nonSelected(TCQuery subQuery, boolean[] isEdgeSelected) {
        boolean shouldSelect = true;
        for (PatternEdge e : subQuery.getEdges()) {
            if (isEdgeSelected[e.getId()]) {
                shouldSelect = false;
                break;
            }
        }
        return shouldSelect;
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
            if (!nonSelected(subQuery, isEdgeSelected))
                continue;

            for (PatternEdge e : subQuery.getEdges())
                isEdgeSelected[e.getId()] = true;
            selectedTCQ.add(subQuery);
        }

        return selectedTCQ;
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

        return selectTCSubQueries(subQueries);
    }
}
