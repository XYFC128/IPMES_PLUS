package org.ipmes.siddhi;

import io.siddhi.query.api.SiddhiApp;
import io.siddhi.query.api.definition.*;
import io.siddhi.query.api.execution.query.Query;
import io.siddhi.query.api.execution.query.input.handler.Window;
import io.siddhi.query.api.execution.query.input.stream.InputStream;
import io.siddhi.query.api.execution.query.selection.Selector;
import io.siddhi.query.api.expression.Expression;
import io.siddhi.query.compiler.SiddhiCompiler;
import org.ipmes.pattern.TemporalRelation;
import org.ipmes.pattern.PatternEdge;
import org.ipmes.pattern.PatternGraph;
import org.ipmes.pattern.PatternNode;

import java.util.ArrayList;

public class SiddhiAppBuilder {
    PatternGraph graph;
    TemporalRelation dependency;
    boolean useRegex;
    SiddhiApp app;

    public SiddhiAppBuilder(PatternGraph patternGraph, TemporalRelation temporalRelation) {
        this.graph = patternGraph;
        this.dependency = temporalRelation;
        this.useRegex = false;
    }

    public SiddhiApp build() {
        this.app = SiddhiApp.siddhiApp("Test");
        defineStreamAndTable();
        this.app.addQuery(genFullMatchQuery());
        return this.app;
    }

    ArrayList<String> genFields() {
        ArrayList<String> fields = new ArrayList<>();
        int numNodes = this.graph.getNodes().size();
        int numEdges = this.graph.getEdges().size();

        for (int i = 0; i < numNodes; ++i) {
            fields.add(String.format("n%d_id", i));
        }
        for (int i = 0; i < numEdges; ++i) {
            fields.add(String.format("e%d_id", i));
        }

        return fields;
    }

    void defineStreamAndTable() {
        StreamDefinition inputStream = StreamDefinition.id("InputStream")
                .attribute("eid", Attribute.Type.STRING)
                .attribute("esig", Attribute.Type.STRING)
                .attribute("start_id", Attribute.Type.STRING)
                .attribute("start_sig", Attribute.Type.STRING)
                .attribute("end_id", Attribute.Type.STRING)
                .attribute("end_sig", Attribute.Type.STRING);
        this.app.defineStream(inputStream);

        StreamDefinition unorderedInputStream = inputStream.clone();
        unorderedInputStream.setId("UnorderedInputStream");
        this.app.defineStream(unorderedInputStream);

        for (PatternEdge e : this.graph.getEdges()) {
            WindowDefinition edgeBuffer = WindowDefinition.id(String.format("E%dBuffer", e.getId()));
            edgeBuffer.attribute("eid", Attribute.Type.STRING)
                    .attribute("esig", Attribute.Type.STRING)
                    .attribute("start_id", Attribute.Type.STRING)
                    .attribute("start_sig", Attribute.Type.STRING)
                    .attribute("end_id", Attribute.Type.STRING)
                    .attribute("end_sig", Attribute.Type.STRING);
            edgeBuffer.window(new Window("time", new Expression[] {Expression.value(10000)}));
            this.app.defineWindow(edgeBuffer);
        }

        ArrayList<String> fields = genFields();

        WindowDefinition candidateWindow = WindowDefinition.id("CandidateTable")
                .window(new Window("time", new Expression[] {Expression.value(10000)}));
        for (String field : fields) {
            candidateWindow.attribute(field, Attribute.Type.STRING);
        }
        this.app.defineWindow(candidateWindow);

        StreamDefinition outputStream = StreamDefinition.id("OutputStream");
        for (String field : fields) {
            outputStream.attribute(field, Attribute.Type.STRING);
        }
        this.app.defineStream(outputStream);
    }

    Query genFullMatchQuery() {
        Query query = Query.query();

        String outputCondition = "";
        ArrayList<String> fields = genFields();
        for (String field : fields) {
            outputCondition += field + " != \"null\" and ";
        }
        outputCondition = outputCondition.substring(0, outputCondition.length() - 5);

        Expression outExpression = SiddhiCompiler.parseExpression(outputCondition);
        query.from(InputStream.stream("InputStream").filter(outExpression))
                .select(Selector.selector().select(Expression.variable("*")))
                .insertInto("OutputStream");
        return query;
    }

    Expression genEdgeCondition(PatternEdge edge) {
        PatternNode startNode = this.graph.getNode(edge.getStartId());
        PatternNode endNode = this.graph.getNode(edge.getEndId());
        String strCondition;
        if (this.useRegex) {
            strCondition = String.format(
                    "regex:matches(\"%s\", esig) and regex:matches(\"%s\", start_sig) and regex:matches(\"%s\", end_sig)",
                    edge.getSignature(),
                    startNode.getSignature(),
                    endNode.getSignature()
            );
        } else {
            strCondition = String.format(
                    "esig == \"%s\" and start_sig == \"%s\" and end_sig == \"%s\"",
                    edge.getSignature(),
                    startNode.getSignature(),
                    endNode.getSignature()
            );
        }
        return SiddhiCompiler.parseExpression(strCondition);
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


}
