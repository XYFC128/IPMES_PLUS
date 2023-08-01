package org.ipmes;

import io.siddhi.core.event.Event;
import io.siddhi.core.stream.output.StreamCallback;

import java.util.ArrayList;
import java.util.HashMap;

public class TCQueryOutputCallback extends StreamCallback {
    TCQuery query;
    PatternGraph patternGraph;
    Join join;
    HashMap<Integer, Integer> node2FieldIdx;
    public TCQueryOutputCallback(TCQuery query, PatternGraph patternGraph, Join join) {
        this.query = query;
        this.patternGraph = patternGraph;
        this.join = join;
        this.node2FieldIdx = new HashMap<>();

        ArrayList<PatternNode> nodes = query.getNodes();
        for (int i = 0; i < nodes.size(); ++i) {
            node2FieldIdx.put(nodes.get(i).getId(), i);
        }
    }

    static Integer parseTimestamp(String tsStr) {
        float toNum = Float.parseFloat(tsStr);
        return Math.round(toNum * 1000);
    }

    ArrayList<DataEdge> toMatchResult(Event e) {
        Object[] data = e.getData();
        int numNodes = this.query.numNodes();
        ArrayList<PatternEdge> patternEdges = this.query.getEdges();

        ArrayList<DataEdge> res = new ArrayList<>();
        for (int i = 0; i < patternEdges.size(); ++i) {
            Integer ts = parseTimestamp((String)data[numNodes + i * 2]);
            String eid = (String)data[numNodes + i * 2 + 1];
            PatternEdge matched = patternEdges.get(i);
            Integer startIdx = this.node2FieldIdx.get(matched.getStartId());
            String startMatch = (String)data[startIdx];
            Integer endIdx = this.node2FieldIdx.get(matched.getEndId());
            String endMatch = (String)data[endIdx];

            res.add(new DataEdge(
                    Integer.parseInt(eid),
                    ts,
                    Integer.parseInt(startMatch),
                    Integer.parseInt(endMatch),
                    matched
            ));
        }
        return res;
    }

    @Override
    public void receive(Event[] events) {
        for (Event e : events) {
            ArrayList<DataEdge> res = toMatchResult(e);
            this.join.addMatchResult(res, this.query.getId());
        }
    }
}
