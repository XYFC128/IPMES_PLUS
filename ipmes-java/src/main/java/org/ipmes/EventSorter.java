package org.ipmes;

import org.ipmes.decomposition.TCQuery;
import org.ipmes.pattern.PatternEdge;
import java.util.regex.Pattern;

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
    ArrayList<Pattern> regexPatterns;
    public EventSorter(ArrayList<TCQuery> tcQueries, boolean useRegex) {
        this.totalOrder = new ArrayList<>();
        for (TCQuery q : tcQueries) {
            totalOrder.addAll(q.getEdges());
        }
        this.useRegex = useRegex;
        if (useRegex)
            compileRegex();
    }

    public void compileRegex() {
        this.regexPatterns = new ArrayList<>();
        for (PatternEdge edge : this.totalOrder) {
            this.regexPatterns.add(Pattern.compile(edge.getSignature()));
        }
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
    public ArrayList<Object[]> rearrangeToEventData(ArrayList<EventEdge> events) {
        ArrayList<Object[]> sorted = new ArrayList<>(events.size());

        for (int i = 0; i < this.totalOrder.size(); ++i) {
            int patternId = totalOrder.get(i).getId();
            for (EventEdge event : events) {
                if (match(i, event))
                    sorted.add(new Object[]{
                            event.timestamp,
                            patternId,
                            event.edgeId,
                            event.startId,
                            event.endId
                    });
            }
        }

        return sorted;
    }

    /**
     * Compare the signatures.
     * @param ord use i-th pattern in total order
     * @param eventEdge the event edge
     * @return true if the signatures match, false otherwise
     */
    boolean match(int ord, EventEdge eventEdge) {
        if (this.useRegex)
            return regexPatterns.get(ord).matcher(eventEdge.signature).find();
        return totalOrder.get(ord).getSignature().equals(eventEdge.signature);
    }
}
