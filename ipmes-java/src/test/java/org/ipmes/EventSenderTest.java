package org.ipmes;

import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class EventSenderTest {
    @Test
    public void testParseTimestamp() {
        assertEquals(1234L, EventSender.parseTimestamp("1.234"));
        assertEquals(1000L, EventSender.parseTimestamp("1.0"));
        assertEquals(2147483648L, EventSender.parseTimestamp("2147483.648"));
    }
}
