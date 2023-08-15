package org.ipmes.match;

import java.util.*;

public class LiteMatchResult {
    public static int MAX_NUM_NODES;
    HashSet<MatchEdge> matchEdges;
    long[] nodeIdMap;
    int numNodes;
    long earliestTime;

    public LiteMatchResult() {
        this.nodeIdMap = new long[MAX_NUM_NODES];
        Arrays.fill(nodeIdMap, -1);
        this.matchEdges = new HashSet<>();
        this.earliestTime = Long.MAX_VALUE;
    }

    public LiteMatchResult(LiteMatchResult other) {
        this.nodeIdMap = Arrays.copyOf(other.nodeIdMap, MAX_NUM_NODES);
        this.numNodes = other.numNodes;
        this.matchEdges = (HashSet<MatchEdge>) other.matchEdges.clone();
        this.earliestTime = other.earliestTime;
    }

    public boolean contains(MatchEdge edge) {
        return this.matchEdges.contains(edge);
    }

    public boolean hasNodeConflict(MatchEdge m) {
        int startPatternId = m.matched.getStartId();
        int endPatternId = m.matched.getEndId();
        if (nodeIdMap[startPatternId] > 0 && nodeIdMap[startPatternId] != m.startId)
            return true;
        if (nodeIdMap[endPatternId] > 0 && nodeIdMap[endPatternId] != m.endId)
            return true;
        return false;
    }

    void addNodeId(long nodeId, int matchedId) {
        if (nodeIdMap[matchedId] == -1)
            ++numNodes;
        nodeIdMap[matchedId] = nodeId;
    }

    public LiteMatchResult cloneAndAdd(MatchEdge m) {
        LiteMatchResult res = new LiteMatchResult(this);
        res.matchEdges.add(m);
        res.addNodeId(m.startId, m.matched.getStartId());
        res.addNodeId(m.endId, m.matched.getEndId());
        res.earliestTime = Math.min(res.earliestTime, m.timestamp);
        return res;
    }

    public long getEarliestTime() {
        return this.earliestTime;
    }

    public MatchResult toMatchResult() {
        MatchResult res = new MatchResult();
        for (MatchEdge match : this.matchEdges)
            res.addMatchEdge(match);
        return res;
    }

    @Override
    public boolean equals(Object obj) {
        if (obj == this)
            return true;
        if (!(obj instanceof LiteMatchResult))
            return false;
        LiteMatchResult other = (LiteMatchResult) obj;
        return this.matchEdges.equals(other.matchEdges);
    }
}
