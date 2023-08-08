package org.ipmes.match;

import org.ipmes.pattern.PatternEdge;

/**
 * Match edge is an edge in streaming data graph. Also, it's an edge in a partial
 * match result, so there must be a pattern edge that matched the data edge.
 */
public class MatchEdge {
    Integer dataId;
    public Integer getDataId() {
        return this.dataId;
    }

    long timestamp;
    public long getTimestamp() {
        return this.timestamp;
    }

    Integer startId;
    public Integer getStartId() {
        return this.startId;
    }

    Integer endId;
    public Integer getEndId() {
        return this.endId;
    }

    PatternEdge matched;

    /**
     * Get the pattern edge this match edge matched with.
     * @return the pattern edge
     */
    public PatternEdge getMatched() {
        return this.matched;
    }

    public Integer matchId() {
        return this.matched.getId();
    }

    public MatchEdge(Integer dataId, long timestamp, Integer startId, Integer endId, PatternEdge matched) {
        this.dataId = dataId;
        this.timestamp = timestamp;
        this.startId = startId;
        this.endId = endId;
        this.matched = matched;
    }

    public Integer[] getEndpoints() {
        return new Integer[] {this.startId, this.endId};
    }

    @Override
    public String toString() {
        return this.dataId.toString();
    }

    @Override
    public boolean equals(Object obj) {
        if (obj == this) return true;
        if (!(obj instanceof MatchEdge)) return false;
        MatchEdge other = (MatchEdge) obj;
        return this.dataId.equals(other.dataId);
    }
}
