package org.ipmes;

import java.util.*;

public class TCQuery {

    int size;
    ArrayList<Integer> query;

    public TCQuery(int size, ArrayList<Integer> query) {
        this.size = size;
        this.query = query;
    }

    public static ArrayList<TCQuery> generate_TCQueries(
            DependencyGraph TemporalRelation,
            PatternGraph SpatialRelation,
            ArrayList<Integer> current_path,
            int current_ID) {
        ArrayList<TCQuery> ret = new ArrayList<TCQuery>();
        ArrayList<Integer> children = TemporalRelation.getChildren(current_ID);
        int numChildren = children.size(), current_path_size = current_path.size();
        ArrayList<Integer> ret_path = new ArrayList<Integer>(current_path);
        ret.add(new TCQuery(current_path_size, ret_path));
        for (int i = 0; i < numChildren; i++) {
            int childId = children.get(i);
            for (int j = 0; j < current_path_size; j++) {
                int current_path_edgeId = current_path.get(j);
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
