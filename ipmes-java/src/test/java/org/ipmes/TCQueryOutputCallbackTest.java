package org.ipmes;

import io.siddhi.core.event.Event;
import org.junit.Test;

import java.io.StringReader;
import java.util.ArrayList;
import java.util.List;

import static org.junit.Assert.*;
public class TCQueryOutputCallbackTest {
    @Test
    public void testParseTimestamp() {
        assertEquals(1234, TCQueryOutputCallback.parseTimestamp("1.234").longValue());
        assertEquals(1000, TCQueryOutputCallback.parseTimestamp("1.0").longValue());
        assertEquals(2147483647, TCQueryOutputCallback.parseTimestamp("2147483.647").longValue());
    }

    @Test
    public void testToMatchResult() {
        String nodes = TTPGenerator.genTTP11Nodes();
        String edges = TTPGenerator.genTTP11Edges();
        PatternGraph pattern = PatternGraph.parse(new StringReader(nodes), new StringReader(edges)).get();

        ArrayList<PatternEdge> queryEdges = new ArrayList<>(List.of(pattern.getEdge(0)));
        TCQuery q = new TCQuery(queryEdges);

        TCQueryOutputCallback callback = new TCQueryOutputCallback(q, pattern, null);
        ArrayList<DataEdge> res = callback.toMatchResult(new Event(0, new Object[] {"0", "1", "100.0", "10"}));
        assertArrayEquals(
                new DataEdge[] {new DataEdge(10, 100000, 0, 1, pattern.getEdge(0))},
                res.toArray());
    }
}
