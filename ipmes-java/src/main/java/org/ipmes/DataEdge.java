package org.ipmes;

/**
 * Data edge is an edge in streaming data graph. Also, it's an edge in a partial
 * match result ,so there must be a pattern edge that matched the data edge.
 */
public class DataEdge {
    Integer dataId;
    public Integer getDataId() {
        return this.dataId;
    }

    Integer timestamp;
    public Integer getTimestamp() {
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
     * Get the pattern edge this data edge matched to.
     * @return the pattern edge
     */
    public PatternEdge getMatched() {
        return this.matched;
    }

    DataEdge(Integer dataId, Integer timestamp, Integer startId, Integer endId, PatternEdge matched) {
        this.dataId = dataId;
        this.timestamp = timestamp;
        this.startId = startId;
        this.endId = endId;
        this.matched = matched;
    }

    public Integer[] getEndpoints() {
        return new Integer[] {this.startId, this.endId};
    }
}
