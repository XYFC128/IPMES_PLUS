package org.ipmes;

import org.ipmes.decomposition.TCQuery;
import org.ipmes.join.Join;
import org.ipmes.match.MatchEdge;
import org.ipmes.match.LightMatchResult;
import org.ipmes.pattern.PatternEdge;

import java.util.*;
import java.util.regex.Pattern;

public class TCMatcher {
    ArrayList<PatternEdge> totalOrder;
    boolean useRegex;
    long windowSize;
    ArrayList<Pattern> regexPatterns;
    PriorityQueue<LightMatchResult>[] buffers;
    int[] tcQueryId;
    Join join;
    public TCMatcher(Collection<TCQuery> tcQueries, boolean useRegex, long windowSize, Join join) {
        this.windowSize = windowSize;
        this.join = join;
        this.totalOrder = new ArrayList<>();
        for (TCQuery q : tcQueries) {
            totalOrder.addAll(q.getEdges());
        }

        initBuffers(tcQueries);

        this.useRegex = useRegex;
        if (useRegex)
            compileRegex();
    }

    void initBuffers(Collection<TCQuery> tcQueries) {
        int len = this.totalOrder.size();
        this.buffers = (PriorityQueue<LightMatchResult>[]) new PriorityQueue[len];
        this.tcQueryId = new int[len];

        int cur = 0;
        for (TCQuery q : tcQueries) {
            for (int i = cur; i < cur + q.numEdges(); ++i) {
                this.tcQueryId[i] = q.getId();
                this.buffers[i] = new PriorityQueue<>(Comparator.comparingLong(LightMatchResult::getEarliestTime));
            }
            this.buffers[cur].add(new LightMatchResult());
            cur += q.numEdges();
        }
    }

    public void compileRegex() {
        this.regexPatterns = new ArrayList<>();
        for (PatternEdge edge : this.totalOrder) {
            this.regexPatterns.add(Pattern.compile(edge.getSignature()));
        }
    }

    void clearExpired(int bufferId, long before) {
        PriorityQueue<LightMatchResult> buffer = this.buffers[bufferId];
        while (!buffer.isEmpty() && buffer.peek().getEarliestTime() < before) {
            buffer.poll();
        }
    }

    ArrayList<LightMatchResult> mergeWithBuffer(MatchEdge match, int bufferId) {
        Collection<LightMatchResult> buffer = this.buffers[bufferId];
        ArrayList<LightMatchResult> merged = new ArrayList<>();
        for (LightMatchResult result : buffer) {
            if (result.hasNodeConflict(match) || result.contains(match))
                continue;
            merged.add(result.cloneAndAdd(match));
        }
        return merged;
    }

    void matchAgainst(Collection<EventEdge> events, int ord) {
        final int numEdges = this.totalOrder.size();
        PatternEdge matched = totalOrder.get(ord);
        ArrayList<LightMatchResult> newResults = new ArrayList<>();
        boolean cleared = false;
        for (EventEdge event : events) {
            if (!match(ord, event))
                continue;
            if (!cleared) {
                long windowBound = event.timestamp - windowSize;
                clearExpired(ord, windowBound);
                cleared = true;
            }
            MatchEdge match = new MatchEdge(event, matched);
            newResults.addAll(mergeWithBuffer(match, ord));
        }

        if (ord == numEdges - 1 || tcQueryId[ord] != tcQueryId[ord + 1]) {
            for (LightMatchResult res : newResults)
                join.addMatchResult(res.toMatchResult(), tcQueryId[ord]);
        } else {
            this.buffers[ord + 1].addAll(newResults);
        }
    }

    public void sendAll(Collection<EventEdge> events) {
        for (int i = 0; i < this.totalOrder.size(); ++i) {
            matchAgainst(events, i);
        }
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
