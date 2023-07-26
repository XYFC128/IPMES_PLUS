package org.ipmes;

import java.util.*;

public class TCQuery {

    int id; // id for the TCQuery
    ArrayList<Integer> query;
    PatternGraph SpatialRelation;

    public TCQuery(ArrayList<Integer> query, PatternGraph SpatialRelation) {
        this.query = query;
        this.SpatialRelation = SpatialRelation;
    }

    // getter for query
    public ArrayList<Integer> getQueryEdges() {
        return this.query;
    }

    // return an arraylist contains every share node of the edge in this query
    public ArrayList<Integer> getInQueryDependencies(Integer eid) {
        ArrayList<Integer> ret = new ArrayList<Integer>();
        for (int i : this.query) {
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

        // add the present path to the subquery set
        ArrayList<Integer> ret_path = new ArrayList<Integer>(current_path);
        ret.add(new TCQuery(ret_path, SpatialRelation));

        // deal with every child
        for (Integer childId : children) {
            for (Integer current_path_edgeId : current_path) {
                // checking 1. and 2.
                if (!(SpatialRelation.getSharedNodes(childId, current_path_edgeId).isEmpty()
                        || current_path.contains(childId))) {
                    ArrayList<Integer> new_current_path = new ArrayList<Integer>(current_path);
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
