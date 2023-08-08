package org.ipmes;

import io.siddhi.core.stream.input.InputHandler;

import java.util.ArrayList;

public class EventSender {
    InputHandler inputHandler;
    EventSorter sorter;
    ArrayList<EventEdge> timeBuffer;
    EventSender(InputHandler handler, EventSorter sorter) {
        this.inputHandler = handler;
        this.sorter = sorter;
        this.timeBuffer = new ArrayList<>();
    }

    public void addLine(String line) throws InterruptedException {
        EventEdge event = new EventEdge(line);
        if (!timeBuffer.isEmpty() && event.timestamp != timeBuffer.get(0).timestamp) {
            flushBuffers();
        }
        timeBuffer.add(event);
    }

    public void flushBuffers() throws InterruptedException {
        ArrayList<Object[]> sorted = sorter.rearrangeToEventData(timeBuffer);
        for (Object[] data : sorted)
            inputHandler.send(data);
        timeBuffer.clear();
    }
}
