package org.ipmes;

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
