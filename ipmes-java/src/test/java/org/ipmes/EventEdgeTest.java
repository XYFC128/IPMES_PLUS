package org.ipmes;

import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class EventEdgeTest {
    @Test
    public void testParseTimestamp() {
        assertEquals(1234L, EventEdge.parseTimestamp("1.234"));
        assertEquals(1000L, EventEdge.parseTimestamp("1.0"));
        assertEquals(2147483648L, EventEdge.parseTimestamp("2147483.648"));
    }
}
