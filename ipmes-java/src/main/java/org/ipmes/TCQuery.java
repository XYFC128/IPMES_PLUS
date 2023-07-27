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

    public ArrayList<Integer> getQueryNodes() {
        HashSet<Integer> nodes = new HashSet<>();
        for (Integer eid : this.query) {
            PatternEdge e = SpatialRelation.getEdge(eid);
            nodes.add(e.getStartId());
            nodes.add(e.getEndId());
        }
        return new ArrayList<Integer>(nodes);
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
}
