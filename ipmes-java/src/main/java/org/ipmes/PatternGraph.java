package org.ipmes;

import java.io.BufferedReader;
import java.io.Reader;
import java.util.*;

import org.json.*;

/**
 * PatternGraph represent the spatial relation of a pattern.
 * <p>
 *     The graph is a directional graph. We will re-assign id
 *     for all nodes and edges in the pattern. The id of edges
 *     and nodes is unique. However, the id of an edge may be the same as
 *     a node, so do not mix them together.
 * </p>
 */
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

                edges.add(new PatternEdge(i, signature, nodes.get(startId), nodes.get(endId)));
                line = edgeBuf.readLine();
            }
            edgeBuf.close();
        } catch (Exception e) {
            return Optional.empty();
        }
        return Optional.of(new PatternGraph(nodes, edges));
    }

    public int numNodes() {
        if (this.nodes == null)
            return 0;
        return this.nodes.size();
    }

    public ArrayList<PatternNode> getNodes() {
        return this.nodes;
    }

    public PatternNode getNode(Integer id) {
        return this.nodes.get(id);
    }

    public int numEdges() {
        if (this.edges == null)
            return 0;
        return this.edges.size();
    }

    public ArrayList<PatternEdge> getEdges() {
        return this.edges;
    }

    public PatternEdge getEdge(Integer eid) {
        return this.edges.get(eid);
    }

    public ArrayList<PatternNode> getSharedNodes(Integer patternEdgeId1, Integer patternEdgeId2) {
        ArrayList<PatternNode> shared = new ArrayList<>();
        PatternEdge e1 = getEdge(patternEdgeId1);
        PatternEdge e2 = getEdge(patternEdgeId2);
        Integer start1 = e1.getStartId(), start2 = e2.getStartId();
        Integer end1 = e1.getEndId(), end2 = e2.getEndId();
        if (start1.equals(start2) || start1.equals(end2))
            shared.add(getNode(start1));
        if (end1.equals(start2) || end1.equals(end2))
            shared.add(getNode(end1));
        return shared;
    }
}
