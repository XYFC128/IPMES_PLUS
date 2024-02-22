package org.ipmes.decomposition;

import org.ipmes.EventEdge;

import java.util.*;

public interface TCMatcher {
    /**
     * Send a batch of event into matcher
     * @param events
     */
    void sendAll(ArrayList<EventEdge> events) throws InterruptedException;

    /**
     * @return the number of intermediate match instances
     */
    int getPoolSize();

    /**
     * @return the trigger count of each TC-Query
     */
    ArrayList<long[]> getTriggerCounts();
}
