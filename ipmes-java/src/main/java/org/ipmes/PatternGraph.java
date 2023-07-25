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

    ArrayList<PatternEdge> getEdges() {
        return this.edges;
    }
}
