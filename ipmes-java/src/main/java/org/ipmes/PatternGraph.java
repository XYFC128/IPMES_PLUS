package org.ipmes;

import java.io.BufferedReader;
import java.io.Reader;
import java.util.*;

import org.json.*;

public class PatternGraph {
    ArrayList<PatternNode> nodes;
    ArrayList<PatternEdge> edges;

    public PatternGraph(ArrayList<PatternNode> nodes, ArrayList<PatternEdge> edges) {
        this.nodes = nodes;
        this.edges = edges;
    }

    public static Optional<PatternGraph> parse(Reader nodeReader, Reader edgeReader) {
        ArrayList<PatternNode> nodes = new ArrayList<>();
        ArrayList<PatternEdge> edges = new ArrayList<>();
        HashMap<String, Integer> idConvert = new HashMap<String, Integer>();
        try {
            BufferedReader nodeBuf = new BufferedReader(nodeReader);
            String line = nodeBuf.readLine();
            for (int i = 0; line != null; i++) {
                JSONObject obj = new JSONObject(line);
                JSONObject nodeObj = obj.getJSONObject("node");
                String rawId = nodeObj.getString("id");
                idConvert.put(rawId, i);
                String signature = Preprocess.extractNodeSignature(nodeObj);

                nodes.add(new PatternNode(i, signature));
                line = nodeBuf.readLine();
            }
            nodeBuf.close();

            BufferedReader edgeBuf = new BufferedReader(edgeReader);
            line = edgeBuf.readLine();
            for (int i = 0; line != null; i++) {
                JSONObject obj = new JSONObject(line);
                JSONObject edgeObj = obj.getJSONObject("edge");
                String raw_start = edgeObj.getJSONObject("start").getString("id");
                String raw_end = edgeObj.getJSONObject("end").getString("id");
                String signature = Preprocess.extractEdgeSignature(edgeObj);
                Integer startId = idConvert.get(raw_start);
                Integer endId = idConvert.get(raw_end);

                edges.add(new PatternEdge(i, signature, startId, endId));
                line = edgeBuf.readLine();
            }
            edgeBuf.close();
        } catch (Exception e) {
            return Optional.empty();
        }
        return Optional.of(new PatternGraph(nodes, edges));
    }

    ArrayList<PatternNode> getNodes() {
        return this.nodes;
    }

    PatternNode getNode(Integer id) {
        return this.nodes.get(id);
    }

    ArrayList<PatternEdge> getEdges() {
        return this.edges;
    }

    PatternEdge getEdge(Integer eid) {
        return this.edges.get(eid);
    }

    public ArrayList<Integer> getSharedNodes(Integer eid1, Integer eid2) {
        ArrayList<Integer> shared = new ArrayList<>();
        PatternEdge e1 = this.edges.get(eid1);
        PatternEdge e2 = this.edges.get(eid2);
        Integer start1 = e1.getStartId(), start2 = e2.getStartId();
        Integer end1 = e1.getEndId(), end2 = e2.getEndId();
        if (start1.equals(start2) || start1.equals(end2))
            shared.add(start1);
        if (end1.equals(start2) || end1.equals(end2))
            shared.add(end1);
        return shared;
    }
}
