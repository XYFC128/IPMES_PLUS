package org.ipmes;

import io.siddhi.core.stream.input.InputHandler;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.PriorityQueue;

/**
 * EventSender handles the sending and buffering of events.
 * <p>
 * Since the input event's time is an interval, we duplicate the event into
 * 2 events with the original start time and end time as their timestamp
 * respectively. Because the input is sorted by the start time, we can be
 * sure that the events prior to current start time can be sent.
 * </p>
 * <p>
 * Also, we will re-order and filter the events with the same timestamp
 * by the total order of pattern, so CEP won't need to worry about the
 * out-of-order events.
 * </p>
 */
public class EventSender {
    TCMatcher tcMatcher;
    ArrayList<EventEdge> timeBuffer;
    PriorityQueue<EventEdge> eventPriorityQueue;
    EventSender(TCMatcher matcher) {
        this.tcMatcher = matcher;
        this.timeBuffer = new ArrayList<>();
        this.eventPriorityQueue = new PriorityQueue<>(Comparator.comparingLong(e -> e.timestamp));
    }

    static long parseTimestamp(String tsStr) {
        double toNum = Double.parseDouble(tsStr);
        return Math.round(toNum * 1000);
    }

    /**
     * Sends the given line. The line is preprocessed csv format of the original event data.
     * @param line the csv row
     */
    public void sendLine(String line) throws InterruptedException {
        String[] fields = line.split(",");
        long startTime   = parseTimestamp(fields[0]);
        long endTime     = parseTimestamp(fields[1]);
        String signature = fields[2];
        long edgeId      = Long.parseLong(fields[3]);
        long startId     = Long.parseLong(fields[4]);
        long endId       = Long.parseLong(fields[5]);

        EventEdge startEvent = new EventEdge(startTime, signature, edgeId, startId, endId);
        this.eventPriorityQueue.add(startEvent);
        if (startTime != endTime) {
            EventEdge endEvent = new EventEdge(endTime, signature, edgeId, startId, endId);
            this.eventPriorityQueue.add(endEvent);
        }
        popQueueUntil(startTime);
    }

    /**
     * Send all event in the buffer prior to the given value.
     * @param time the timestamp
     */
    void popQueueUntil(long time) throws InterruptedException {
        while (!eventPriorityQueue.isEmpty() && eventPriorityQueue.peek().timestamp < time) {
            EventEdge event = eventPriorityQueue.poll();
            if (!timeBuffer.isEmpty() && event.timestamp != timeBuffer.get(0).timestamp) {
                flushTimeBuffer();
            }
            timeBuffer.add(event);
        }
    }

    /**
     * Sort the time buffer by total order and send to CEP.
     */
    void flushTimeBuffer() throws InterruptedException {
        tcMatcher.sendAll(timeBuffer);
        timeBuffer.clear();
    }

    /**
     * Flush out all the buffered events.
     */
    public void flushBuffers() throws InterruptedException {
        popQueueUntil(Long.MAX_VALUE);
    }
}
