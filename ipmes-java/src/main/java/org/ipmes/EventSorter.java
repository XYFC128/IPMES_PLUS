package org.ipmes;

import java.util.ArrayList;

public class EventSorter {
    ArrayList<PatternEdge> totalOrder;
    public EventSorter(ArrayList<TCQuery> tcQueries) {
        this.totalOrder = new ArrayList<>();
        for (TCQuery q : tcQueries) {
            totalOrder.addAll(q.getEdges());
        }
    }

    static boolean match(PatternEdge patternEdge, EventEdge eventEdge) {
        String startSig = patternEdge.getStartNode().getSignature();
        String endSig = patternEdge.getEndNode().getSignature();
        return patternEdge.getSignature().equals(eventEdge.edgeSignature) &&
                startSig.equals(eventEdge.startSignature) &&
                endSig.equals(eventEdge.endSignature);
    }

    public ArrayList<EventEdge> rearrange(ArrayList<EventEdge> events) {
        ArrayList<EventEdge> sorted = new ArrayList<>(events.size());

        for (PatternEdge edge : this.totalOrder) {
            for (EventEdge data : events) {
                if (match(edge, data))
                    sorted.add(data);
            }
        }

        return sorted;
    }
}
