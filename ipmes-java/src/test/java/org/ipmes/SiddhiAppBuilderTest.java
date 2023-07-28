package org.ipmes;
import io.siddhi.query.api.SiddhiApp;
import org.junit.Test;

import java.io.StringReader;

import static org.junit.Assert.*;
public class SiddhiAppBuilderTest {
    @Test
    public void test() {
        String nodes = TTPGenerator.genTTP11Nodes();
        String edges = TTPGenerator.genTTP11Edges();
        PatternGraph pattern = PatternGraph.parse(new StringReader(nodes), new StringReader(edges)).get();
        String orels = TTPGenerator.genTTP11Orels();
        DependencyGraph dep = DependencyGraph.parse(new StringReader(orels)).get();
        SiddhiAppBuilder builder = new SiddhiAppBuilder(pattern, dep);
        SiddhiApp app = builder.build();
        System.out.println(app.getExecutionElementList());
    }
}
