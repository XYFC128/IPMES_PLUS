package org.ipmes;

import java.util.*;

public class Decomposition {

    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;

    public Decomposition(DependencyGraph temporalRelation, PatternGraph spatialRelation) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
    }

    // DFS to find out possible TC subqueries
    // constraints:
    // 1. new edge is not in the TC subquery yet
    // 2. new edge need to have share node with TC subquery
    // 3. follow temporal rules
    // so we handle edges in temporal order, then check 1. and 2.

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

    public ArrayList<TCQuery> decompose() {

        /*
         * ///////////////////////////////////////////////
         * DFS to generate TC subqueries
         *////////////////////////////////////////////////

        int numEdges = this.spatialRelation.getEdges().size();
        ArrayList<TCQuery> subQueries = new ArrayList<TCQuery>();
        for (int i = 0; i < numEdges; i++) {
            ArrayList<Integer> tmpList = new ArrayList<Integer>();
            tmpList.add(i);
            subQueries.addAll(generateTCQueries(tmpList, i));
        }

        /*
         * ///////////////////////////////////////////////
         * greedy select longest TC subqueries
         *////////////////////////////////////////////////

        // sort in decreasing size
        subQueries.sort((Q1, Q2) -> (Q2.query.size() - Q1.query.size()));
        // subQueryProvider: index of subqueries
        ArrayList<Integer> subQueryProvider = new ArrayList<Integer>();
        // subQuerySelected: result of greedy select
        ArrayList<Integer> subQuerySelected = new ArrayList<Integer>();
        ArrayList<TCQuery> parseToCEP = new ArrayList<TCQuery>();
        for (int i = 0; i < subQueries.size(); i++) {
            boolean shouldSelect = true;
            for (int j : subQueries.get(i).query) {
                if (subQuerySelected.contains(j)) {
                    shouldSelect = false;
                    break;
                }
            }
            if (shouldSelect) {
                subQueryProvider.add(i);
                subQueries.get(i).setTCQueryID(i);
                subQuerySelected.addAll(subQueries.get(i).query);
                parseToCEP.add(subQueries.get(i));
            }
        }
        return parseToCEP;
    }
}
