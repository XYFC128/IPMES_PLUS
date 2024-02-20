package org.ipmes;

import org.ipmes.event.DefaultEventSender;
import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class DefaultEventSenderTest {
    @Test
    public void testParseTimestamp() {
        assertEquals(1234L, DefaultEventSender.parseTimestamp("1.234"));
        assertEquals(1000L, DefaultEventSender.parseTimestamp("1.0"));
        assertEquals(2147483648L, DefaultEventSender.parseTimestamp("2147483.648"));
    }
}
