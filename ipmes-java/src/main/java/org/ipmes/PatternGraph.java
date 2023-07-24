package org.ipmes;

import java.io.File;
import java.util.*;

import org.json.*;

public class PatternGraph {
    ArrayList<PatternNode> nodes;
    ArrayList<PatternEdge> edges;

    public PatternGraph(ArrayList<PatternNode> nodes, ArrayList<PatternEdge> edges) {
        this.nodes = nodes;
        this.edges = edges;
    }

    public static Optional<PatternGraph> parse(String nodeFile, String edgeFile) {
        ArrayList<PatternNode> nodes = new ArrayList<>();
        ArrayList<PatternEdge> edges = new ArrayList<>();
        try {
            Scanner nodeScanner = new Scanner(new File(nodeFile));
            HashMap<String, Integer> idConvert = new HashMap<String, Integer>();
            for (int i = 0; nodeScanner.hasNextLine(); i++) {
                String line = nodeScanner.nextLine();
                JSONObject obj = new JSONObject(line);
                JSONObject nodeObj = obj.getJSONObject("node");
                String rawId = nodeObj.getString("id");
                idConvert.put(rawId, i);
                String signature = Preprocess.extractNodeSignature(nodeObj);

                nodes.add(new PatternNode(i, signature));
            }
            nodeScanner.close();

            Scanner edgeScanner = new Scanner(new File(edgeFile));
            for (int i = 0; edgeScanner.hasNextLine(); i++) {
                String line = edgeScanner.nextLine();
                JSONObject obj = new JSONObject(line);
                JSONObject edgeObj = obj.getJSONObject("edge");
                String raw_start = edgeObj.getJSONObject("start").getString("id");
                String raw_end = edgeObj.getJSONObject("end").getString("id");
                String signature = Preprocess.extractEdgeSignature(edgeObj);
                Integer startId = idConvert.get(raw_start);
                Integer endId = idConvert.get(raw_end);

                edges.add(new PatternEdge(i, signature, startId, endId));
            }
            edgeScanner.close();
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
