package org.ipmes.siddhi;

import io.siddhi.core.event.Event;
import org.ipmes.TTPGenerator;
import org.ipmes.decomposition.TCQuery;
import org.ipmes.match.MatchEdge;
import org.ipmes.match.MatchResult;
import org.ipmes.pattern.PatternEdge;
import org.ipmes.pattern.PatternGraph;
import org.ipmes.siddhi.TCQueryOutputCallback;
import org.junit.Test;

import java.io.StringReader;
import java.util.ArrayList;
import java.util.List;

import static org.junit.Assert.*;
public class TCQueryOutputCallbackTest {
    @Test
    public void testToMatchResult() {
        String nodes = TTPGenerator.genTTP11Nodes();
        String edges = TTPGenerator.genTTP11Edges();
        PatternGraph pattern = PatternGraph.parse(new StringReader(nodes), new StringReader(edges)).get();

        ArrayList<PatternEdge> queryEdges = new ArrayList<>(List.of(pattern.getEdge(0)));
        TCQuery q = new TCQuery(queryEdges);

        TCQueryOutputCallback callback = new TCQueryOutputCallback(q, pattern, null);
        MatchResult res = callback.toMatchResult(new Event(0, new Object[] {1L, 0L, 100000L, 10L, 0L}));
        assertArrayEquals(
                new MatchEdge[] {new MatchEdge(10, 100000, 0, 1, pattern.getEdge(0))},
                res.matchEdges().toArray());
    }
}
