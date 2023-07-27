package org.ipmes;

import io.siddhi.query.api.SiddhiApp;
import io.siddhi.query.compiler.SiddhiCompiler;
import org.junit.Test;

import java.io.StringReader;
import java.util.ArrayList;

import static org.junit.Assert.assertNotNull;

public class DecompositionTest {
    @Test
    public void testDecomposition() {
        String nodes = TTPGenerator.genTTP11Nodes();
        String edges = TTPGenerator.genTTP11Edges();
        PatternGraph pattern = PatternGraph.parse(new StringReader(nodes), new StringReader(edges)).get();
        String orels = TTPGenerator.genTTP11Orels();
        DependencyGraph dep = DependencyGraph.parse(new StringReader(orels)).get();

        Decomposition d = new Decomposition(dep, pattern);
        ArrayList<TCQuery> queries = d.decompose();

        TCSiddhiAppGenerator gen = new TCSiddhiAppGenerator(pattern, dep, queries);
        String appStr = gen.generate();
        System.out.println(gen.generate());
        SiddhiApp app = SiddhiCompiler.parse(appStr);
        assertNotNull(app);
    }
}
