package org.ipmes.match;

import org.ipmes.pattern.PatternEdge;

/**
 * Match edge is an edge in streaming data graph. Also, it's an edge in a partial
 * match result, so there must be a pattern edge that matched the data edge.
 */
public class MatchEdge {
    long dataId;
    public long getDataId() {
        return this.dataId;
    }

    long timestamp;
    public long getTimestamp() {
        return this.timestamp;
    }

    long startId;
    public long getStartId() {
        return this.startId;
    }

    long endId;
    public long getEndId() {
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

    public MatchEdge(long dataId, long timestamp, long startId, long endId, PatternEdge matched) {
        this.dataId = dataId;
        this.timestamp = timestamp;
        this.startId = startId;
        this.endId = endId;
        this.matched = matched;
    }

    public Long[] getEndpoints() {
        return new Long[] {this.startId, this.endId};
    }

    @Override
    public String toString() {
        return Long.toString(this.dataId);
    }

    @Override
    public boolean equals(Object obj) {
        if (obj == this) return true;
        if (!(obj instanceof MatchEdge)) return false;
        MatchEdge other = (MatchEdge) obj;
        return this.dataId == other.dataId;
    }
}
