package org.ipmes;

import io.siddhi.query.api.SiddhiApp;
import org.junit.Test;

import java.io.StringReader;
import java.util.ArrayList;
import java.util.Arrays;

import io.siddhi.query.compiler.SiddhiCompiler;

import static org.junit.Assert.*;

public class SiddhiQueryTest {
    SiddhiAppGenerator createSimpleGenerator() {
        ArrayList<PatternNode> nodes = new ArrayList<>();
        nodes.add(new PatternNode(0, "aaa"));
        nodes.add(new PatternNode(1, "bbb"));

        ArrayList<PatternEdge> edges = new ArrayList<>();
        edges.add(new PatternEdge(0, "edge", nodes.get(0), nodes.get(1)));

        PatternGraph pattern = new PatternGraph(nodes, edges);
        DependencyGraph dependency = new DependencyGraph(
                new ArrayList<>(Arrays.asList(new ArrayList<>(), new ArrayList<>())),
                new ArrayList<>(Arrays.asList(new ArrayList<>(), new ArrayList<>()))
        );
        return new SiddhiAppGenerator(pattern, dependency, false);
    }

    SiddhiAppGenerator createTTP11Generator() {
        String nodes = TTPGenerator.genTTP11Nodes();
        String edges = TTPGenerator.genTTP11Edges();
        PatternGraph pattern = PatternGraph.parse(new StringReader(nodes), new StringReader(edges)).get();
        String orels = TTPGenerator.genTTP11Orels();
        DependencyGraph dep = DependencyGraph.parse(new StringReader(orels)).get();
        return new SiddhiAppGenerator(pattern, dep, false);
    }

    @Test
    public void genHeader() {
        SiddhiAppGenerator generator = createSimpleGenerator();

        assertEquals("define Stream InputStream (eid string, esig string, start_id string, start_sig string, end_id string, end_sig string);\n" +
                "define Stream UnorderedInputStream (eid string, esig string, start_id string, start_sig string, end_id string, end_sig string);\n" +
                "define Window E0Buffer (eid string, esig string, start_id string, start_sig string, end_id string, end_sig string) time(10 sec);\n" +
                "define Window CandidateTable (n0_id string, n1_id string, e0_id string) time(10 sec);\n" +
                "@sink(type=\"log\") define Stream OutputStream (n0_id string, n1_id string, e0_id string);\n",
                generator.genStreamAndWindowDefinition());
    }

    @Test
    public void genOutputConditionQuery() {
        SiddhiAppGenerator generator = createSimpleGenerator();

        assertEquals("from CandidateTable[n0_id != \"null\" and n1_id != \"null\" and e0_id != \"null\"]\n" +
                        "select *\n" +
                        "insert into OutputStream;\n",
                generator.genFullMatchQuery());
    }

    @Test
    public void genEdgeCondition() {
        SiddhiAppGenerator generator = createSimpleGenerator();
        ArrayList<PatternNode> nodes = generator.graph.getNodes();
        assertEquals("esig == \"edge\" and start_sig == \"aaa\" and end_sig == \"bbb\"",
                generator.genEdgeCondition(new PatternEdge(0, "edge", nodes.get(0), nodes.get(1))));
    }

    @Test
    public void genSelectExpression() {
        SiddhiAppGenerator generator = createSimpleGenerator();
        PatternEdge edge = generator.graph.getEdge(0);

        assertEquals("\"null\" as n0_id, \"null\" as n1_id, \"null\" as e0_id",
                generator.genSelectExpression(edge, "\"null\"", "\"null\"", "\"null\"", "null"));
        assertEquals("a as n0_id, b as n1_id, c as e0_id",
                generator.genSelectExpression(edge, "a", "b", "c", "null"));
    }

    @Test
    public void shouldCompile1() {
        SiddhiAppGenerator gen = createSimpleGenerator();
        SiddhiApp app = SiddhiCompiler.parse(gen.generate());
        assertNotNull(app);
    }

    @Test
    public void shouldCompile2() {
        SiddhiAppGenerator gen = createTTP11Generator();
        SiddhiApp app = SiddhiCompiler.parse(gen.generate());
        assertNotNull(app);
    }
}
