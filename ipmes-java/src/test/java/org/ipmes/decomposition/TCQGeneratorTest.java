package org.ipmes.decomposition;

import io.siddhi.core.SiddhiAppRuntime;
import io.siddhi.core.SiddhiManager;
import io.siddhi.core.event.Event;
import io.siddhi.core.stream.input.InputHandler;
import io.siddhi.core.stream.output.StreamCallback;
import org.ipmes.TTPGenerator;
import org.ipmes.pattern.TemporalRelation;
import org.ipmes.pattern.PatternGraph;
import org.ipmes.siddhi.TCSiddhiAppGenerator;
import org.junit.Test;

import java.io.StringReader;
import java.util.ArrayList;

import static org.junit.Assert.*;
import static org.junit.Assert.assertArrayEquals;

public class TCQGeneratorTest {
    @Test
    public void testTCQueryStructure() {
        String nodes = TTPGenerator.genTTP11Nodes();
        String edges = TTPGenerator.genTTP11Edges();
        PatternGraph pattern = PatternGraph.parse(new StringReader(nodes), new StringReader(edges)).get();
        String orels = TTPGenerator.genTTP11Orels();
        TemporalRelation dep = TemporalRelation.parse(new StringReader(orels)).get();

        TCQGenerator d = new TCQGenerator(dep, pattern);
        ArrayList<TCQuery> queries = d.decompose();

        assertFalse(queries.isEmpty());
        TCQuery q = queries.get(0);
        assertArrayEquals(pattern.getNodes().toArray(), q.getNodes().toArray());
        assertArrayEquals(pattern.getEdges().toArray(), q.getEdges().toArray());
    }

    @Test
    public void testDecomposition() throws Exception {
        // init TTP11
        String nodes = TTPGenerator.genTTP11Nodes();
        String edges = TTPGenerator.genTTP11Edges();
        PatternGraph pattern = PatternGraph.parse(new StringReader(nodes), new StringReader(edges)).get();
        String orels = TTPGenerator.genTTP11Orels();
        TemporalRelation dep = TemporalRelation.parse(new StringReader(orels)).get();

        // Decomposition
        TCQGenerator d = new TCQGenerator(dep, pattern);
        TCSiddhiAppGenerator gen = new TCSiddhiAppGenerator(pattern, dep, d.decompose(), Long.MAX_VALUE);

        // Generate CEP app and runtime
        String appStr = gen.generate();
        System.out.println(appStr);
        SiddhiManager siddhiManager = new SiddhiManager();
        SiddhiAppRuntime runtime = siddhiManager.createSiddhiAppRuntime(appStr);

        final int[] resultCount = {0};
        runtime.addCallback("TC0Output", new StreamCallback() {
            @Override
            public void receive(Event[] events) {
                for (Event e : events) {
                    resultCount[0]++;
                    System.out.println(e.toString());
                }
                //To convert and print event as a map
                //EventPrinter.print(toMap(events));
            }
        });

        InputHandler inputHandler = runtime.getInputHandler("InputStream");
        runtime.start();
        // inputHandler.send(new Object[]{"ts", "match_id", "eid", "start_id", "end_id"});
        inputHandler.send(new Object[]{0L, 0, 0L, 0L, 1L});
        inputHandler.send(new Object[]{100L, 1, 1L, 1L, 2L});
        inputHandler.send(new Object[]{200L, 2, 2L, 3L, 2L});
        inputHandler.send(new Object[]{300L, 3, 3L, 4L, 2L});

        runtime.shutdown();
        siddhiManager.shutdown();
        assertEquals(1, resultCount[0]);
    }
}
