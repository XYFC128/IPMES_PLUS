package org.ipmes;

import java.util.*;

public class TCQuery {

    // we can later sort quiries by size to greedy choose
    int size;
    ArrayList<Integer> query;

    public TCQuery(int size, ArrayList<Integer> query) {
        this.size = size;
        this.query = query;
    }

    // DFS to find out possible TC subqueries
    // constraints:
    // 1. new edge is not in the TC subquery yet
    // 2. new edge need to have share node with TC subquery
    // 3. follow temporal rules
    // so we handle edges in temporal order, then check 1. and 2.
    public static ArrayList<TCQuery> generate_TCQueries(
            DependencyGraph TemporalRelation,
            PatternGraph SpatialRelation,
            ArrayList<Integer> current_path,
            int current_ID) {
        // the TC subquery set to be return
        ArrayList<TCQuery> ret = new ArrayList<TCQuery>();
        // extract children
        ArrayList<Integer> children = TemporalRelation.getChildren(current_ID);

        int numChildren = children.size(), current_path_size = current_path.size();

        // add the present path to the subquery set
        ArrayList<Integer> ret_path = new ArrayList<Integer>(current_path);
        ret.add(new TCQuery(current_path_size, ret_path));

        // deal with every child
        for (int i = 0; i < numChildren; i++) {
            int childId = children.get(i);
            for (int j = 0; j < current_path_size; j++) {
                int current_path_edgeId = current_path.get(j);
                // checking 1. and 2.
                if (!(SpatialRelation.getSharedNodes(childId, current_path_edgeId).isEmpty()
                        || current_path.contains(childId))) {
                    ArrayList<Integer> new_current_path = current_path;
                    new_current_path.add(childId);
                    ret.addAll(generate_TCQueries(
                            TemporalRelation,
                            SpatialRelation,
                            new_current_path,
                            childId));
                    continue;
                }
            }
        }
        return ret;
    }
}
