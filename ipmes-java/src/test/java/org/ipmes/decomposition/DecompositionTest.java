package org.ipmes.decomposition;

import io.siddhi.core.SiddhiAppRuntime;
import io.siddhi.core.SiddhiManager;
import io.siddhi.core.event.Event;
import io.siddhi.core.stream.input.InputHandler;
import io.siddhi.core.stream.output.StreamCallback;
import org.ipmes.TTPGenerator;
import org.ipmes.decomposition.Decomposition;
import org.ipmes.decomposition.TCQuery;
import org.ipmes.pattern.DependencyGraph;
import org.ipmes.pattern.PatternGraph;
import org.ipmes.siddhi.TCSiddhiAppGenerator;
import org.junit.Test;

import java.io.StringReader;
import java.util.ArrayList;

import static org.junit.Assert.*;
import static org.junit.Assert.assertArrayEquals;

public class DecompositionTest {
    @Test
    public void testTCQueryStructure() {
        String nodes = TTPGenerator.genTTP11Nodes();
        String edges = TTPGenerator.genTTP11Edges();
        PatternGraph pattern = PatternGraph.parse(new StringReader(nodes), new StringReader(edges)).get();
        String orels = TTPGenerator.genTTP11Orels();
        DependencyGraph dep = DependencyGraph.parse(new StringReader(orels)).get();

        Decomposition d = new Decomposition(dep, pattern);
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
        DependencyGraph dep = DependencyGraph.parse(new StringReader(orels)).get();

        // Decomposition
        Decomposition d = new Decomposition(dep, pattern);
        TCSiddhiAppGenerator gen = new TCSiddhiAppGenerator(pattern, dep, d.decompose());

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
        // inputHandler.send(new Object[]{"ts", "eid", "esig", "start_id", "start_sig", "end_id", "end_sig"});
        inputHandler.send(new Object[]{"ts", "0", "fork",   "0", "Process::hello.sh",                     "1", "Process::hello.sh"});
        inputHandler.send(new Object[]{"ts", "1", "execve", "1", "Process::hello.sh",                     "2", "Process::journalctl"});
        inputHandler.send(new Object[]{"ts", "2", "load",   "3", "Artifact::file::/usr/bin/journalctl",   "2", "Process::journalctl"});
        inputHandler.send(new Object[]{"ts", "3", "open",   "4", "Artifact::directory::/var/log/journal", "2", "Process::journalctl"});

        inputHandler.send(new Object[]{"ts", "4", "fork",   "5", "Process::hello.sh",                      "6", "Process::hello.sh"});
        inputHandler.send(new Object[]{"ts", "5", "execve", "7", "Process::hello.sh",                      "8", "Process::journalctl"});
        inputHandler.send(new Object[]{"ts", "6", "load",   "9", "Artifact::file::/usr/bin/journalctl",    "10", "Process::journalctl"});
        inputHandler.send(new Object[]{"ts", "7", "open",   "11", "Artifact::directory::/var/log/journal", "12", "Process::journalctl"});

        runtime.shutdown();
        siddhiManager.shutdown();
        assertEquals(1, resultCount[0]);
    }
}
