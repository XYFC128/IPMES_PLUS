package org.ipmes.match;

import java.util.*;

/**
 * The partial match result for the entire pattern.
 */
public class MatchResult {
    int hash;
    HashMap<Integer, MatchEdge> results;
    BitSet resultContains;
    MatchResult next;
    long earliestTime;
    long latestTime;

    public MatchResult() {
        this.hash = 0;
        this.earliestTime = Long.MAX_VALUE;
        this.latestTime = Long.MIN_VALUE;
        this.results = new HashMap<>();
        this.resultContains = new BitSet();
        this.next = null;
    }

    /**
     * Calculate a^n mod p
     */
    static long pow(long a, long n, long p) {
        long res = 1;
        long base = a;
        while (n > 0) {
            if ((n & 1) == 1)
                res = (res * base) % p;
            base = (base * base) % p;
            n >>= 1;
        }
        return res;
    }

    public void addMatchEdge(MatchEdge m) {
        Integer matchId = m.matched.getId();
        this.hash += (int)(m.getDataId() * pow(7, matchId, Integer.MAX_VALUE) % Integer.MAX_VALUE);
        this.earliestTime = Math.min(this.earliestTime, m.timestamp);
        this.latestTime = Math.max(this.latestTime, m.timestamp);
        this.results.put(matchId, m);
        this.resultContains.set(matchId);
    }

    public boolean hasShareEdge(MatchResult other) {
        return this.resultContains.intersects(other.resultContains);
    }

    public MatchResult merge(MatchResult other) {
        MatchResult res = new MatchResult();
        res.results.putAll(this.results);
        res.results.putAll(other.results);
        res.hash = (int)(((long)this.hash + (long)other.hash) % Integer.MAX_VALUE);
        res.resultContains.or(this.resultContains);
        res.resultContains.or(other.resultContains);
        res.earliestTime = Math.min(this.earliestTime, other.earliestTime);
        res.latestTime = Math.max(this.latestTime, other.latestTime);
        return res;
    }

    public Collection<MatchEdge> matchEdges() {
        return this.results.values();
    }

    public MatchEdge get(Integer patternEdgeId) {
        return this.results.get(patternEdgeId);
    }

    public boolean containsPattern(Integer patternEdgeId) {
        return this.results.containsKey(patternEdgeId);
    }

    public int size() {
        return this.results.size();
    }

    public long getEarliestTime() {
        return this.earliestTime;
    }

    public long getLatestTime() {
        return this.latestTime;
    }

    public MatchResult getNext() {
        return this.next;
    }

    public void setNext(MatchResult next) {
        this.next = next;
        return;
    }

    /*
    Return false if there is a input node matches 2 or more pattern node.
     */
    public boolean checkNodeUniqueness() {
        HashMap<Long, Integer> nodeMap = new HashMap<>();
        for (MatchEdge edge : this.results.values()) {
            int startPattern = edge.matched.getStartId();
            int endPattern = edge.matched.getEndId();
            if (nodeMap.getOrDefault(edge.startId, startPattern) != startPattern)
                return false;
            if (nodeMap.getOrDefault(edge.endId, endPattern) != endPattern)
                return false;
            nodeMap.put(edge.startId, startPattern);
            nodeMap.put(edge.endId, endPattern);
        }
        return true;
    }

    public FullMatch toFullMatch() {
        FullMatch res = new FullMatch(this.results.size());
        for (Map.Entry<Integer, MatchEdge> entry : this.results.entrySet())
            res.set(entry.getKey(), entry.getValue().getDataId());
        return res;
    }

    @Override
    public int hashCode() {
        return this.hash;
    }

    @Override
    public boolean equals(Object obj) {
        if (obj == this)
            return true;
        if (!(obj instanceof MatchResult))
            return false;
        MatchResult other = (MatchResult) obj;
        if (this.results.size() != other.results.size())
            return false;
        for (Integer pattern_id : this.results.keySet()) {
            MatchEdge e1 = this.results.get(pattern_id);
            MatchEdge e2 = other.results.get(pattern_id);
            if (e2 == null || e1.dataId != e2.dataId || e1.timestamp != e2.timestamp)
                return false;
        }
        return true;
    }
}
