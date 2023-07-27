package org.ipmes;

import java.util.ArrayList;
import java.util.HashMap;

public class TCSiddhiAppGenerator {
    PatternGraph patternGraph;
    DependencyGraph dependencyGraph;
    ArrayList<TCQuery> tcQueries;
    public TCSiddhiAppGenerator(PatternGraph patternGraph, DependencyGraph dependencyGraph, ArrayList<TCQuery> tcQueries) {
        this.patternGraph = patternGraph;
        this.dependencyGraph = dependencyGraph;
        this.tcQueries = tcQueries;
    }

    public String generate() {
        String app = "@App:name(\"SiddhiApp\")\n";
        app += genStreamDefinition();
        for (TCQuery q : this.tcQueries)
            app += genTCPatternQuery(q);
        return app;
    }

    String genTCQueryStreamDefinition(TCQuery q) {
        String def = String.format("define stream TC%dOutput (", q.getTCQueryID());
        ArrayList<Integer> edges = q.getQueryEdges();

        String fields = "";
        for (Integer i : q.getQueryNodes()) {
            fields += String.format("n%d_id string, ", i);
        }
        for (Integer i : q.getQueryEdges()) {
            fields += String.format("e%1$d_ts string, e%1$d_id string, ", i);
        }

        if (!fields.isEmpty())
            fields = fields.substring(0, fields.length() - 2);

        return def + fields + ");\n";
    }

    String genStreamDefinition() {
        String def = "define Stream InputStream (timestamp string, eid string, esig string, start_id string, start_sig string, end_id string, end_sig string);\n";
        for (TCQuery q : this.tcQueries)
            def += genTCQueryStreamDefinition(q);
        return def;
    }

    String genSharedNodeConditions(PatternEdge edge, HashMap<Integer, String> prefixNodes) {
        ArrayList<String> conditions = new ArrayList<>();
        String startPrefix = prefixNodes.get(edge.getStartId());
        String endPrefix = prefixNodes.get(edge.getEndId());
        if (startPrefix != null)
            conditions.add("start_id == " + startPrefix);
        if (endPrefix != null)
            conditions.add("end_id == " + endPrefix);
        return String.join(" and ", conditions);
    }

    String genEdgeCondition(PatternEdge edge, HashMap<Integer, String> prefixNodes) {
        String sharedNodeCondition = genSharedNodeConditions(edge, prefixNodes);

        PatternNode startNode = this.patternGraph.getNode(edge.getStartId());
        PatternNode endNode = this.patternGraph.getNode(edge.getEndId());
        String signatureCondition = String.format(
            "esig == \"%s\" and start_sig == \"%s\" and end_sig == \"%s\"",
            edge.getSignature(),
            startNode.getSignature(),
            endNode.getSignature()
        );
        if (sharedNodeCondition.isEmpty())
            return signatureCondition;
        return sharedNodeCondition + " and " + signatureCondition;
    }

    String genSelectExpression(TCQuery q, HashMap<Integer, String> prefixNodes) {
        ArrayList<String> expr = new ArrayList<>();

        ArrayList<Integer> nodes = q.getQueryNodes();
        for (Integer nid : nodes) {
            expr.add(String.format("%s as n%d_id", prefixNodes.get(nid), nid));
        }

        for (Integer eid : q.getQueryEdges()) {
            expr.add(String.format("e%1$d.eid as e%1$d_id", eid));
        }

        return "select " + String.join(", ", expr);
    }

    String genTCPatternQuery(TCQuery q) {
        String query = "from ";

        HashMap<Integer, String> prefixNodes = new HashMap<>();
        for (Integer eid : q.getQueryEdges()) {
            if (!prefixNodes.isEmpty())
                query += "  -> ";
            PatternEdge edge = patternGraph.getEdge(eid);
            query += String.format("every(e%d = InputStream[%s])\n",
                    edge.getId(), genEdgeCondition(edge, prefixNodes));
            prefixNodes.put(edge.getStartId(), String.format("e%d.start_id", edge.getId()));
            prefixNodes.put(edge.getEndId(), String.format("e%d.end_id", edge.getId()));
        }
        query += genSelectExpression(q, prefixNodes);
        query += String.format("\ninsert into TC%dOutput;\n", q.getTCQueryID());

        return query;
    }
}
