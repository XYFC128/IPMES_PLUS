package org.ipmes;

import java.util.ArrayList;
import java.util.BitSet;
import java.util.Collection;
import java.util.HashMap;

public class MatchResult {
    int hash;
    HashMap<Integer, MatchEdge> results;
    BitSet resultContains;

    public MatchResult() {
        this.hash = 0;
        this.results = new HashMap<>();
        this.resultContains = new BitSet();
    }

    public void addMatchEdge(MatchEdge m) {
        Integer matchId = m.matched.getId();
        this.hash += m.getDataId() * matchId;
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
        res.hash = this.hash + other.hash;
        res.resultContains.or(this.resultContains);
        res.resultContains.or(other.resultContains);
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
        return this.results.equals(other.results);
    }
}
