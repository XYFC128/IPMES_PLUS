package org.ipmes.decomposition;

import org.ipmes.EventEdge;
import org.ipmes.decomposition.TCQuery;
import org.ipmes.join.Join;
import org.ipmes.match.MatchEdge;
import org.ipmes.match.LiteMatchResult;
import org.ipmes.pattern.PatternEdge;

import java.util.*;
import com.google.re2j.Pattern;

/**
 * This class is responsible for matching TC-Queries.
 */
public class TCMatcher {
    ArrayList<PatternEdge> totalOrder;
    boolean useRegex;
    long windowSize;
    ArrayList<Pattern> regexPatterns;
    ArrayDeque<LiteMatchResult>[] buffers;
    long[] triggerCount;
    int[] tcQueryId;
    Join join;
    int poolSize;
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
        this.poolSize = 0;
    }

    void initBuffers(Collection<TCQuery> tcQueries) {
        int len = this.totalOrder.size();
        this.buffers = (ArrayDeque<LiteMatchResult>[]) new ArrayDeque[len];
        this.tcQueryId = new int[len + 1];
        tcQueryId[len] = -1;

        this.triggerCount = new long[len];
        Arrays.fill(this.triggerCount, 0);

        int cur = 0;
        for (TCQuery q : tcQueries) {
            for (int i = cur; i < cur + q.numEdges(); ++i) {
                this.tcQueryId[i] = q.getId();
                this.buffers[i] = new ArrayDeque<>();
            }
            this.buffers[cur].add(new LiteMatchResult());
            cur += q.numEdges();
        }
    }

    public int getPoolSize() {
        return this.poolSize;
    }

    public void compileRegex() {
        this.regexPatterns = new ArrayList<>();
        for (PatternEdge edge : this.totalOrder) {
            this.regexPatterns.add(Pattern.compile(edge.getSignature()));
        }
    }

    void clearExpired(int bufferId, long before) {
        ArrayDeque<LiteMatchResult> buffer = this.buffers[bufferId];
        int cleared = 0;
        while (!buffer.isEmpty() && buffer.peekFirst().getEarliestTime() < before) {
            buffer.pollFirst();
            ++cleared;
        }
        this.poolSize -= cleared;
    }

    void putIntoBuffer(int bufferId, Collection<LiteMatchResult> newEntries) {
        this.buffers[bufferId].addAll(newEntries);
        int len = newEntries.size();
        if (len > 0) {
            this.poolSize += len;
        }
    }

    ArrayList<LiteMatchResult> mergeWithBuffer(MatchEdge match, int bufferId) {
        Collection<LiteMatchResult> buffer = this.buffers[bufferId];
        ArrayList<LiteMatchResult> merged = new ArrayList<>();
        for (LiteMatchResult result : buffer) {
            if (result.hasNodeConflict(match) || result.contains(match))
                continue;
            merged.add(result.cloneAndAdd(match));
        }
        return merged;
    }

    void matchAgainst(Collection<EventEdge> events, int ord) {
        if (this.buffers[ord].isEmpty())
            return;

        final int numEdges = this.totalOrder.size();
        PatternEdge matched = totalOrder.get(ord);
        ArrayList<LiteMatchResult> newResults = new ArrayList<>();
        for (EventEdge event : events) {
            if (!match(ord, event))
                continue;

            MatchEdge match = new MatchEdge(event, matched);
            newResults.addAll(mergeWithBuffer(match, ord));
        }

        this.triggerCount[ord] += newResults.size();

        if (tcQueryId[ord] != tcQueryId[ord + 1]) {
            for (LiteMatchResult res : newResults)
                if (res.checkNodeUniqueness())
                    join.addMatchResult(res.toMatchResult(), tcQueryId[ord]);
        } else {
            putIntoBuffer(ord + 1, newResults);
        }
    }

    public void sendAll(Collection<EventEdge> events) {
        if (events.isEmpty())
            return;
        EventEdge first = events.iterator().next();
        long windowBound = first.timestamp - windowSize;
        for (int i = 0; i < this.totalOrder.size(); ++i) {
            clearExpired(i, windowBound);
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

    public ArrayList<long[]> getTriggerCounts() {
        ArrayList<long[]> triggerCounts = new ArrayList<>();
        int start = 0;
        for (int i = 1; i < this.tcQueryId.length; ++i) {
            if (this.tcQueryId[i] == this.tcQueryId[i - 1])
                continue;
            triggerCounts.add(Arrays.copyOfRange(this.triggerCount, start, i));
            start = i;
        }
        return triggerCounts;
    }
}
