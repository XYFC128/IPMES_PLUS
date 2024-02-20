package org.ipmes.event;

public interface EventSender {
    void sendLine(String line) throws InterruptedException;
    void flushBuffers() throws InterruptedException;
}
