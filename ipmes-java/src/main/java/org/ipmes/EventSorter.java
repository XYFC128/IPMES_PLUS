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

    static boolean match(PatternEdge edge, Object[] eventData) {
        String startSig = edge.getStartNode().getSignature();
        String endSig = edge.getEndNode().getSignature();
        return edge.getSignature().equals(eventData[2]) &&
                startSig.equals(eventData[4]) && endSig.equals(eventData[6]);
    }

    public ArrayList<Object[]> sort(ArrayList<Object[]> events) {
        ArrayList<Object[]> sorted = new ArrayList<>(events.size());

        for (PatternEdge edge : this.totalOrder) {
            for (Object[] data : events) {
                if (match(edge, data))
                    sorted.add(data);
            }
        }

        return sorted;
    }
}
