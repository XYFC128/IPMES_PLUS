package org.ipmes;

import java.util.*;

public class Decomposition {

    DependencyGraph TemporalRelation;
    PatternGraph SpatialRelation;

    public Decomposition(DependencyGraph TemporalRelation, PatternGraph SpatialRelation) {
        this.TemporalRelation = TemporalRelation;
        this.SpatialRelation = SpatialRelation;
    }

    // DFS to find out possible TC subqueries
    // constraints:
    // 1. new edge is not in the TC subquery yet
    // 2. new edge need to have share node with TC subquery
    // 3. follow temporal rules
    // so we handle edges in temporal order, then check 1. and 2.

    private ArrayList<TCQuery> generate_TCQueries(ArrayList<Integer> current_path, int current_ID) {
        // the TC subquery set to be return
        ArrayList<TCQuery> ret = new ArrayList<TCQuery>();
        // extract children
        ArrayList<Integer> children = this.TemporalRelation.getChildren(current_ID);

        // add the present path to the subquery set
        ArrayList<Integer> ret_path = new ArrayList<Integer>(current_path);
        ret.add(new TCQuery(ret_path, this.SpatialRelation));

        // deal with every child
        for (Integer childId : children) {
            for (Integer current_path_edgeId : current_path) {
                // checking 1. and 2.
                if (!(this.SpatialRelation.getSharedNodes(childId, current_path_edgeId).isEmpty()
                        || current_path.contains(childId))) {
                    ArrayList<Integer> new_current_path = new ArrayList<Integer>(current_path);
                    new_current_path.add(childId);
                    ret.addAll(generate_TCQueries(new_current_path, childId));
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

        int numEdges = this.SpatialRelation.getEdges().size();
        ArrayList<TCQuery> subQueries = new ArrayList<TCQuery>();
        for (int i = 0; i < numEdges; i++) {
            ArrayList<Integer> tmpList = new ArrayList<Integer>();
            tmpList.add(i);
            subQueries.addAll(generate_TCQueries(tmpList, i));
        }

        /*
         * ///////////////////////////////////////////////
         * greedy select longest TC subqueries
         *////////////////////////////////////////////////

        // sort in decreasing size
        subQueries.sort((Q1, Q2) -> (Q2.query.size() - Q1.query.size()));
        // subQ_provider: index of subqueries
        ArrayList<Integer> subQ_provider = new ArrayList<Integer>();
        // subQ_selected: result of greedy select
        ArrayList<Integer> subQ_selected = new ArrayList<Integer>();
        ArrayList<TCQuery> ParseToCEP = new ArrayList<TCQuery>();
        for (int i = 0; i < subQueries.size(); i++) {
            boolean shouldSelect = true;
            for (int j : subQueries.get(i).query) {
                if (subQ_selected.contains(j)) {
                    shouldSelect = false;
                    break;
                }
            }
            if (shouldSelect) {
                subQ_provider.add(i);
                subQueries.get(i).setTCQueryID(i);
                subQ_selected.addAll(subQueries.get(i).query);
                ParseToCEP.add(subQueries.get(i));
            }
        }
        return ParseToCEP;
    }
}
