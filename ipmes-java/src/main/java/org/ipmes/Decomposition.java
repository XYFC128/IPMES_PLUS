package org.ipmes;

import java.util.*;

public class Decomposition {

    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;

    public Decomposition(DependencyGraph temporalRelation, PatternGraph spatialRelation) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
    }

    /**
     * DFS on {@link DependencyGraph} to find out all possible TC sub-queries
     * starting at
     * the given node in the graph.
     * <p>
     * When encounter a new node, we check the following constraints:
     * <ol>
     * <li>new edge is not in the TC sub-query yet</li>
     * <li>new edge need to have share node with TC sub-query</li>
     * <li>follow temporal rules</li>
     * </ol>
     * If all checks passed, create a new TC sub-query and append to the results.
     * </p>
     * 
     * @param currentPath current traversed path
     * @param currentId   the starting node
     * @return all possible TC sub-queries starting at the node
     */
    private ArrayList<TCQuery> generateTCQueries(ArrayList<Integer> currentPath, int currentId) {
        // the TC subquery set to be return
        ArrayList<TCQuery> ret = new ArrayList<TCQuery>();
        // extract children
        ArrayList<Integer> children = this.temporalRelation.getChildren(currentId);

        // add the present path to the subquery set
        ArrayList<Integer> retPath = new ArrayList<Integer>(currentPath);
        ret.add(new TCQuery(retPath, this.spatialRelation));

        // deal with every child
        for (Integer childId : children) {
            for (Integer currentPathEdgeId : currentPath) {
                // checking 1. and 2.
                if (!(this.spatialRelation.getSharedNodes(childId, currentPathEdgeId).isEmpty()
                        || currentPath.contains(childId))) {
                    ArrayList<Integer> newCurrentPath = new ArrayList<Integer>(currentPath);
                    newCurrentPath.add(childId);
                    ret.addAll(generateTCQueries(newCurrentPath, childId));
                    continue;
                }
            }
        }
        return ret;
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
        subQueries.sort((Q1, Q2) -> (Q2.edges.size() - Q1.edges.size()));
        // subQueryProvider: index of subqueries
        ArrayList<Integer> subQueryProvider = new ArrayList<Integer>();
        // subQuerySelected: result of greedy select
        ArrayList<Integer> subQuerySelected = new ArrayList<Integer>();
        ArrayList<TCQuery> parseToCEP = new ArrayList<TCQuery>();
        for (int i = 0; i < subQueries.size(); i++) {
            boolean shouldSelect = true;
            for (int j : subQueries.get(i).edges) {
                if (subQuerySelected.contains(j)) {
                    shouldSelect = false;
                    break;
                }
            }
            if (shouldSelect) {
                subQueryProvider.add(i);
                subQueries.get(i).setTCQueryID(i);
                subQuerySelected.addAll(subQueries.get(i).edges);
                parseToCEP.add(subQueries.get(i));
            }
        }
        return parseToCEP;
    }

    /**
     * Decompose the possibly non-TC pattern into TC-Queries
     *
     * @return TC-Queries
     */
    public ArrayList<TCQuery> decompose() {
        // DFS to generate TC sub-queries
        int numEdges = this.spatialRelation.getEdges().size();
        ArrayList<TCQuery> subQueries = new ArrayList<TCQuery>();
        for (int i = 0; i < numEdges; i++) {
            ArrayList<Integer> tmpList = new ArrayList<Integer>();
            tmpList.add(i);
            subQueries.addAll(generateTCQueries(tmpList, i));
        }

        return selectTCSubQueries(subQueries);
    }
}
