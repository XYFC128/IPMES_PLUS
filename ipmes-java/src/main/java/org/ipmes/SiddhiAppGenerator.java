package org.ipmes;


public class SiddhiAppGenerator {
    PatternGraph graph;
    DependencyGraph dependency;
    boolean useRegex;

    public SiddhiAppGenerator(PatternGraph patternGraph, DependencyGraph dependencyGraph, boolean useRegex) {
        this.graph = patternGraph;
        this.dependency = dependencyGraph;
        this.useRegex = useRegex;
    }

    public String generate() {
        String app = "@App:name(\"SiddhiApp\")\n";
        app += genStreamAndWindowDefinition();
        app += genFullMatchQuery();
        for (PatternEdge e : this.graph.getEdges())
            app += genEdgeRule(e);

        return app;
    }

    String genTableFields() {
        String fields = "";

        int numNodes = this.graph.getNodes().size();
        int numEdges = this.graph.getEdges().size();

        for (int i = 0; i < numNodes; ++i) {
            fields += String.format("n%d_id string, ", i);
        }
        for (int i = 0; i < numEdges; ++i) {
            fields += String.format("e%d_id string, ", i);
        }

        if (fields.endsWith(", ")) {
            fields = fields.substring(0, fields.length() - 2);
        }
        return fields;
    }

    String genStreamAndWindowDefinition() {
        String def = "define Stream InputStream (eid string, esig string, start_id string, start_sig string, end_id string, end_sig string);\n" +
                "define Stream UnorderedInputStream (eid string, esig string, start_id string, start_sig string, end_id string, end_sig string);\n";

        for (PatternEdge e : this.graph.getEdges()) {
            def += String.format("define Window E%dBuffer (eid string, esig string, start_id string, start_sig string, end_id string, end_sig string) time(10 sec);\n", e.getId());
        }

        String fields = genTableFields();
        def += String.format("define Window CandidateTable (%s) time(10 sec);\n", fields);
        def += String.format("@sink(type=\"log\") define Stream OutputStream (%s);\n", fields);

        return def;
    }

    String genFullMatchQuery() {
        String outputCondition = "";

        for (PatternNode nd : this.graph.getNodes()) {
            outputCondition += String.format("n%d_id != \"null\" and ", nd.getId());
        }
        for (PatternEdge e : this.graph.getEdges()) {
            outputCondition += String.format("e%d_id != \"null\" and ", e.getId());
        }

        if (outputCondition.endsWith("and ")) {
            outputCondition = outputCondition.substring(0, outputCondition.length() - 5);
        }
        String query = String.format( "from CandidateTable[%s]\n" +
                "select *\n" +
                "insert into OutputStream;\n", outputCondition);
        return query;
    }

    String genEdgeCondition(PatternEdge edge) {
        PatternNode startNode = this.graph.getNode(edge.getStartId());
        PatternNode endNode = this.graph.getNode(edge.getEndId());
        if (this.useRegex) {
            return String.format(
                    "regex:matches(\"%s\", esig) and regex:matches(\"%s\", start_sig) and regex:matches(\"%s\", end_sig)",
                    edge.getSignature(),
                    startNode.getSignature(),
                    endNode.getSignature()
            );
        } else {
            return String.format(
                    "esig == \"%s\" and start_sig == \"%s\" and end_sig == \"%s\"",
                    edge.getSignature(),
                    startNode.getSignature(),
                    endNode.getSignature()
            );
        }
    }

    String genDependencyCondition(PatternEdge edge) {
        String condition = "";
        for (int parentId : this.dependency.getParents(edge.getId())) {
            if (parentId == -1)
                continue;
            condition += String.format("t.e%d_id != \"null\" and ", parentId);
        }
        return condition;
    }

    String genNodeSelectExpr(PatternEdge edge, String start_id, String end_id, String source) {
        String selectExpr = "";
        int numNodes = this.graph.getNodes().size();
        for (int i = 0; i < numNodes; ++i) {
            String field_name = String.format("n%d_id", i);
            if (i == edge.getStartId()) {
                selectExpr += String.format("%s as %s", start_id, field_name);
            } else if (i == edge.getEndId()) {
                selectExpr += String.format("%s as %s", end_id, field_name);
            } else if (!source.equals("null")) {
                selectExpr += String.format("%s.%s", source, field_name);
            } else {
                selectExpr += "\"null\" as " + field_name;
            }
            selectExpr += ", ";
        }
        return selectExpr;
    }

    String genEdgeSelectExpr(PatternEdge edge, String eid, String source) {
        String selectExpr = "";
        int numEdges = this.graph.getEdges().size();
        for (int i = 0; i < numEdges; ++i) {
            String field_name = String.format("e%d_id", i);
            if (i == edge.getId()) {
                selectExpr += String.format("%s as %s", eid, field_name);
            } else if (!source.equals("null")) {
                selectExpr += String.format("%s.%s", source, field_name);
            } else {
                selectExpr += "\"null\" as " + field_name;
            }
            selectExpr += ", ";
        }
        return selectExpr;
    }

    String genSelectExpression(PatternEdge edge, String start_id, String end_id, String eid, String source) {
        // (start, end, normal, this, normal)
        String selectExpr = genNodeSelectExpr(edge, start_id, end_id, source) +
                genEdgeSelectExpr(edge, eid, source);
        if (!selectExpr.isEmpty())
            selectExpr = selectExpr.substring(0, selectExpr.length() - 2); // remove ", "

        return selectExpr;
    }

    String genNewCandidateQuery(PatternEdge edge, String edgeCondition) {
        String query = "";
        if (!this.dependency.getParents(edge.getId()).isEmpty()) {
            String selectExpr = genSelectExpression(edge, "start_id", "end_id", "eid", "null");
            query = String.format("from InputStream[%1$s]\n" +
                    "select %2$s\n" +
                    "insert into CandidateTable;\n" +
                    "\n" +
                    "from E%3$sBuffer\n" +
                    "select %2$s\n" +
                    "insert into CandidateTable;\n",
                    edgeCondition, selectExpr, edge.getId());
        }
        return query;
    }

    String genJoinExpression(String source, PatternEdge edge, String depCondition) {
        String edgeField = String.format("e%d_id", edge.getId());
        String startField = String.format("n%d_id", edge.getStartId());
        String endField = String.format("n%d_id", edge.getEndId());
        String selectExpr = genSelectExpression(edge, "s.start_id", "s.end_id", "s.eid", "t");

        String expr = String.format("from %1$s as s join\n" +
                "    CandidateTable as t\n" +
                "    on t.%2$s == \"null\" and %3$s\n" +
                "        ((t.%4$s == s.start_id and t.%5$s == s.end_id) or\n" +
                "        (t.%4$s == s.start_id and t.%5$s == \"null\") or\n" +
                "        (t.%4$s == \"null\" and t.%5$s == s.end_id))\n" +
                "select %6$s\n" +
                "insert into CandidateTable;\n",
                source, edgeField, depCondition, startField, endField, selectExpr);
        return expr;
    }

    String genEdgeRule(PatternEdge edge) {
        String rule = "";

        String edgeCondition = genEdgeCondition(edge);
        String depCondition = genDependencyCondition(edge);

        rule += genNewCandidateQuery(edge, edgeCondition);
        rule += String.format("from UnorderedInputStream[%s]\n" +
                "select *\n" +
                "insert into E%dBuffer;\n",
                edgeCondition, edge.getId());

        rule += genJoinExpression(
                String.format("InputStream[%s]", edgeCondition),
                edge,
                depCondition
        );
        rule += genJoinExpression(
                String.format("E%dBuffer", edge.getId()),
                edge,
                depCondition
        );

        return rule;
    }
}
