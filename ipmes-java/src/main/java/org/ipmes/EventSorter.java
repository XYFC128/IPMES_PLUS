package org.ipmes;

import org.ipmes.decomposition.TCQuery;
import org.ipmes.pattern.PatternEdge;

import java.util.ArrayList;

/**
 * EventSorter is used to rearrange the same timestamp events. Since they arrive at the same time,
 * they could in any order. We rearrange them by their total order. The total order is an order of
 * pattern edges that satisfy the time constrains in the decomposed TC-Queries. However, the order
 * between TC-Queries are not considered.
 */
public class EventSorter {
    ArrayList<PatternEdge> totalOrder;
    boolean useRegex;
    public EventSorter(ArrayList<TCQuery> tcQueries, boolean useRegex) {
        this.totalOrder = new ArrayList<>();
        for (TCQuery q : tcQueries) {
            totalOrder.addAll(q.getEdges());
        }
        this.useRegex = useRegex;
    }

    /**
     * Rearrange the events by the total order. Note that this function no just sort the
     * events, it may duplicate the events matching multiple pattern edges, so <b> the output
     * array length may be longer.</b>
     * <p>
     * This may result in multiple occurrences of the same event edge in a match result.
     * Therefore, the uniqueness check in the match result is important.
     * </p>
     * @param events the events with the same timestamp
     * @return the sorted events
     */
    public ArrayList<EventEdge> rearrange(ArrayList<EventEdge> events) {
        ArrayList<EventEdge> sorted = new ArrayList<>(events.size());

        for (PatternEdge pattern : this.totalOrder) {
            for (EventEdge event : events) {
                if (match(pattern, event))
                    sorted.add(event);
            }
        }

        return sorted;
    }

    /**
     * Compare the signatures.
     * TODO: support regex matching
     * @param patternEdge the pattern edge
     * @param eventEdge the event edge
     * @return true if the signatures match, false otherwise
     */
    static boolean match(PatternEdge patternEdge, EventEdge eventEdge) {
        String startSig = patternEdge.getStartNode().getSignature();
        String endSig = patternEdge.getEndNode().getSignature();
        return patternEdge.getSignature().equals(eventEdge.edgeSignature) &&
                startSig.equals(eventEdge.startSignature) &&
                endSig.equals(eventEdge.endSignature);
    }
}
