package org.ipmes;

import io.siddhi.core.stream.input.InputHandler;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.PriorityQueue;

public class EventSender {
    InputHandler inputHandler;
    EventSorter sorter;
    ArrayList<EventEdge> timeBuffer;
    PriorityQueue<EventEdge> eventPriorityQueue;
    EventSender(InputHandler handler, EventSorter sorter) {
        this.inputHandler = handler;
        this.sorter = sorter;
        this.timeBuffer = new ArrayList<>();
        this.eventPriorityQueue = new PriorityQueue<>((Comparator.comparingLong(e -> e.timestamp)));
    }

    static long parseTimestamp(String tsStr) {
        double toNum = Double.parseDouble(tsStr);
        return Math.round(toNum * 1000);
    }

    public void addLine(String line) throws InterruptedException {
        String[] fields = line.split(",");
        long startTime   = parseTimestamp(fields[0]);
        long endTime     = parseTimestamp(fields[1]);
        String signature = fields[2];
        long edgeId      = Long.parseLong(fields[3]);
        long startId     = Long.parseLong(fields[4]);
        long endId       = Long.parseLong(fields[5]);

        if (startTime == endTime) {
            EventEdge event = new EventEdge(startTime, signature, edgeId, startId, endId);
            this.eventPriorityQueue.add(event);
        } else {
            EventEdge startEvent = new EventEdge(startTime, signature, edgeId, startId, endId);
            this.eventPriorityQueue.add(startEvent);
            EventEdge endEvent = new EventEdge(endTime, signature, edgeId, startId, endId);
            this.eventPriorityQueue.add(endEvent);
        }
        popQueueUntil(startTime);
    }

    void popQueueUntil(long time) throws InterruptedException {
        while (!eventPriorityQueue.isEmpty() && eventPriorityQueue.peek().timestamp < time) {
            EventEdge event = eventPriorityQueue.poll();
            if (!timeBuffer.isEmpty() && event.timestamp != timeBuffer.get(0).timestamp) {
                flushTimeBuffer();
            }
            timeBuffer.add(event);
        }
    }

    void flushTimeBuffer() throws InterruptedException {
        ArrayList<Object[]> sorted = sorter.rearrangeToEventData(timeBuffer);
        for (Object[] data : sorted)
            inputHandler.send(data);
        timeBuffer.clear();
    }

    public void flushBuffers() throws InterruptedException {
        popQueueUntil(Long.MAX_VALUE);
    }
}
