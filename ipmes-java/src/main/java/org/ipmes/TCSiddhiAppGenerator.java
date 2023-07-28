package org.ipmes;

import java.util.ArrayList;
import java.util.HashMap;

/**
 * A generator that generates Siddhi app in a String to match given TC-Queries.
 * <p>
 * The generated Siddhi app utilize Siddhi's temporal pattern syntax to match
 * the temporal order of the TC-Queries. The output of a TC-Query q will be
 * a stream called TC{q}Output. The field of output stream contains every matched
 * node id in that query, timestamp and id of matched edge.
 *
 * @see TCQuery
 */
public class TCSiddhiAppGenerator {
    PatternGraph patternGraph;
    DependencyGraph dependencyGraph;
    ArrayList<TCQuery> tcQueries;
    boolean useRegex;
    public TCSiddhiAppGenerator(PatternGraph patternGraph, DependencyGraph dependencyGraph, ArrayList<TCQuery> tcQueries) {
        this.patternGraph = patternGraph;
        this.dependencyGraph = dependencyGraph;
        this.tcQueries = tcQueries;
        this.useRegex = false;
    }

    public void setUseRegex(boolean flag) {
        this.useRegex = flag;
    }

    /**
     * Start the generating process
     * @return A Siddhi app that can be fed into Siddhi compiler and runtime
     */
    public String generate() {
        String app = "@App:name(\"SiddhiApp\")\n";
        app += genStreamDefinition();
        for (TCQuery q : this.tcQueries)
            app += genTCPatternQuery(q);
        return app;
    }

    /**
     * Generate the output stream definition of given TC-Query.
     * <p>
     * The fields of output stream are:
     * <ol>
     *     <li>n{i}_id string: the matched data node id of the pattern node i.</li>
     *     <li>e{j}_id string: the matched data edge id of the pattern edge j.</li>
     *     <li>e{j}_ts string: the matched data edge timestamp of the pattern edge j.</li>
     * </ol>
     * The order of fields is like: n{i}_id, ..., e{j}_id, e{j}_ts, e{k}_id, e{k}_ts, ...
     *
     * @param q the TC-Query want to generate for
     * @return the generated stream definition, with an end-of-line character at the end
     */
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

    /**
     * Generate the definition of all the streams we need, including the input stream
     * and the output stream of each TC-Queries.
     *
     * @return the stream definitions, with an end-of-line character at the end
     */
    String genStreamDefinition() {
        String def = "define Stream InputStream (timestamp string, eid string, esig string, start_id string, start_sig string, end_id string, end_sig string);\n";
        for (TCQuery q : this.tcQueries)
            def += genTCQueryStreamDefinition(q);
        return def;
    }

    /**
     * Generate the condition expression to check the shared node relation between given
     * edge and its prefix in a TC-Query.
     * <p>
     *     We only need to know the nodes in its prefix and their owner (who the node
     *     belongs to, that is, the start node or end node of what edge). So, we can
     *     check for each endpoints of the pattern edge if it already has a owner in
     *     the prefix, and generate the conditional expression requiring they must be
     *     the same.
     * </p>
     *
     * @param edge the pattern we want to generate for
     * @param prefixNodes a map recording the owner of each pattern nodes in the prefix
     * @return the condition expression to check the shared node relation
     */
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

    /**
     * Generate the shared node condition and signature match expression for a pattern
     * edge. If enable regex matching, it will treat the extracted signature as a
     * regular expression and use regex matching to match the data edges.
     *
     * @param edge the pattern edge we generate for
     * @param prefixNodes same as the prefixNodes in the parameter of genSharedNodeConditions
     * @return the combined edge condition
     */
    String genEdgeCondition(PatternEdge edge, HashMap<Integer, String> prefixNodes) {
        String sharedNodeCondition = genSharedNodeConditions(edge, prefixNodes);

        PatternNode startNode = this.patternGraph.getNode(edge.getStartId());
        PatternNode endNode = this.patternGraph.getNode(edge.getEndId());
        String signatureCondition;
        if (this.useRegex) {
            signatureCondition = String.format(
                    "regex:matches(\"%s\", esig) and regex:matches(\"%s\", start_sig) and regex:matches(\"%s\", end_sig)",
                    edge.getSignature(),
                    startNode.getSignature(),
                    endNode.getSignature()
            );
        } else {
            signatureCondition = String.format(
                    "esig == \"%s\" and start_sig == \"%s\" and end_sig == \"%s\"",
                    edge.getSignature(),
                    startNode.getSignature(),
                    endNode.getSignature()
            );
        }
        if (sharedNodeCondition.isEmpty())
            return signatureCondition;
        return sharedNodeCondition + " and " + signatureCondition;
    }

    /**
     * Generate the expression to select the required field of the match result of a TC-Query
     * to insert into its output stream.
     *
     * @param q the TC-Query
     * @param nodeOwner a map recording the owner of each pattern nodes
     * @return the select expression
     */
    String genSelectExpression(TCQuery q, HashMap<Integer, String> nodeOwner) {
        ArrayList<String> expr = new ArrayList<>();

        ArrayList<Integer> nodes = q.getQueryNodes();
        for (Integer nid : nodes) {
            expr.add(String.format("%s as n%d_id", nodeOwner.get(nid), nid));
        }

        for (Integer eid : q.getQueryEdges()) {
            expr.add(String.format("e%1$d.timestamp as e%1$d_ts, e%1$d.eid as e%1$d_id", eid));
        }

        return "select " + String.join(", ", expr);
    }

    /**
     * Generate the Siddhi pattern query syntax for a TC-Query.
     * <p>
     *     The generated query will match the result from InputStream and insert
     *     a new entry in the output stream of the TC-Query.
     * </p>
     * @param q the TC-Query
     * @return the Siddhi pattern query
     */
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
